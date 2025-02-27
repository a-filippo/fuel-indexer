use crate::{
    assets, defaults, utils::update_test_manifest_asset_paths, TestError, WORKSPACE_ROOT,
};
use actix_service::Service;
use actix_web::test;
use axum::routing::Router;
use fuel_indexer::IndexerService;
use fuel_indexer_api_server::api::WebApi;
use fuel_indexer_database::IndexerConnectionPool;
use fuel_indexer_lib::{
    config::{DatabaseConfig, IndexerConfig, WebApiConfig},
    defaults::SERVICE_REQUEST_CHANNEL_SIZE,
    manifest::Manifest,
    utils::{derive_socket_addr, ServiceRequest},
};
use fuel_indexer_postgres;
use fuels::{
    macros::abigen,
    prelude::{
        setup_single_asset_coins, setup_test_client, AssetId, Bech32ContractId, Contract,
        LoadConfiguration, Provider, TxParameters, WalletUnlocked, DEFAULT_COIN_AMOUNT,
    },
    test_helpers::Config,
};
use hyper::Error;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::{pool::Pool, Connection, Executor, PgConnection, Postgres};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
};

use tokio::{
    sync::mpsc::{channel, Receiver},
    task::JoinHandle,
    time::{sleep, Duration},
};
use tracing_subscriber::filter::EnvFilter;

abigen!(Contract(
    name = "FuelIndexerTest",
    abi = "packages/fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test-abi.json"
));

pub struct TestPostgresDb {
    pub db_name: String,
    pub url: String,
    pub pool: Pool<Postgres>,
    server_connection_str: String,
}

pub struct IndexingTestComponents {
    pub node: JoinHandle<Result<(), ()>>,
    pub db: TestPostgresDb,
    pub service: IndexerService,
    pub manifest: Manifest,
}

impl Drop for IndexingTestComponents {
    fn drop(&mut self) {
        // FIXME: cancel the tokio task
        self.node.abort();
    }
}

pub struct WebTestComponents {
    pub node: JoinHandle<Result<(), ()>>,
    pub db: TestPostgresDb,
    pub service: IndexerService,
    pub manifest: Manifest,
    pub app: Router,
    #[allow(unused)]
    pub rx: Receiver<ServiceRequest>,
    pub server: JoinHandle<Result<(), Error>>,
    pub client: reqwest::Client,
}

pub async fn mock_request(path: &str) {
    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(test_web::app(contract)).await;
    let req = test::TestRequest::post().uri(path).to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
}

pub async fn setup_indexing_test_components(
    config: Option<IndexerConfig>,
) -> IndexingTestComponents {
    let node = tokio::spawn(setup_example_test_fuel_node());
    let db = TestPostgresDb::new().await.unwrap();
    let mut service = indexer_service_postgres(Some(&db.url), config).await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    service
        .register_indexer_from_manifest(
            manifest.clone(),
            fuel_indexer_lib::defaults::REMOVE_DATA,
        )
        .await
        .unwrap();

    IndexingTestComponents {
        node,
        db,
        service,
        manifest,
    }
}

pub async fn setup_web_test_components(
    config: Option<IndexerConfig>,
) -> WebTestComponents {
    let node = tokio::spawn(setup_example_test_fuel_node());
    let db = TestPostgresDb::new().await.unwrap();
    let mut service = indexer_service_postgres(Some(&db.url), config.clone()).await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    service
        .register_indexer_from_manifest(
            manifest.clone(),
            fuel_indexer_lib::defaults::REMOVE_DATA,
        )
        .await
        .unwrap();

    let (app, rx) = api_server_app_postgres(Some(&db.url), config).await;

    let server = axum::Server::bind(&WebApiConfig::default().into())
        .serve(app.clone().into_make_service());
    let server = tokio::spawn(server);

    let client = reqwest::ClientBuilder::new()
        .pool_max_idle_per_host(0)
        .build()
        .unwrap();

    WebTestComponents {
        node,
        db,
        service,
        manifest,
        app,
        rx,
        server,
        client,
    }
}

impl TestPostgresDb {
    pub async fn new() -> Result<Self, TestError> {
        // Generate a random string to serve as a unique name for a temporary database
        let rng = thread_rng();
        let db_name: String = rng
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        // The server connection string serves as a way to connect directly to the Postgres server.
        // Example database URL: postgres://postgres:my-secret@localhost:5432
        let connection_config: DatabaseConfig = std::env::var("DATABASE_URL")
            .unwrap_or(defaults::POSTGRES_URL.to_string())
            .parse()?;
        let server_connection_str = connection_config.to_string();

        let DatabaseConfig::Postgres {
            user,
            password,
            host,
            port,
            ..
        } = connection_config;
        let test_db_config = DatabaseConfig::Postgres {
            user,
            password,
            host,
            port,
            database: db_name.clone(),
            verbose: "true".to_string(),
        };

        // Connect directly to the Postgres server and create a database with the unique string
        let mut conn = PgConnection::connect(server_connection_str.as_str()).await?;

        conn.execute(format!(r#"CREATE DATABASE "{}""#, &db_name).as_str())
            .await?;

        // Instantiate a pool so that it can be stored in the struct for use in the tests
        let pool =
            match IndexerConnectionPool::connect(&test_db_config.clone().to_string())
                .await
            {
                Ok(pool) => match pool {
                    IndexerConnectionPool::Postgres(p) => {
                        let mut conn = p.acquire().await?;

                        fuel_indexer_postgres::run_migration(&mut conn).await?;
                        p
                    }
                },
                Err(e) => return Err(TestError::PoolCreationError(e)),
            };

        Ok(Self {
            db_name,
            url: test_db_config.to_string(),
            pool,
            server_connection_str,
        })
    }

    async fn teardown(&mut self) -> Result<(), TestError> {
        let mut conn = PgConnection::connect(&self.server_connection_str).await?;

        // Drop all connections to the test database so that resources are cleaned up
        conn.execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                AND pid <> pg_backend_pid()
                "#,
                self.db_name
            )
            .as_str(),
        )
        .await?;

        // Forcefully drop the database. Connections should have been cleaned up by
        // this point as we've awaited the prior query, but let's just do it by force.
        conn.execute(
            format!(
                r#"DROP DATABASE IF EXISTS "{}" WITH (FORCE);"#,
                self.db_name
            )
            .as_str(),
        )
        .await?;

        Ok(())
    }
}

impl Drop for TestPostgresDb {
    fn drop(&mut self) {
        // drop() cannot be async. Thus, we create a blocking thread
        // to await the teardown operation for the database.
        std::thread::scope(|s| {
            s.spawn(|| {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                runtime.block_on(self.teardown()).unwrap();
            });
        });
    }
}

pub fn tx_params() -> TxParameters {
    let gas_price = 0;
    let gas_limit = 1_000_000;
    let byte_price = 0;
    TxParameters::new(gas_price, gas_limit, byte_price)
}

pub async fn setup_test_fuel_node(
    wallet_path: PathBuf,
    contract_bin_path: Option<PathBuf>,
    host_str: Option<String>,
) -> Result<(), ()> {
    let filter = match std::env::var_os("RUST_LOG") {
        Some(_) => EnvFilter::try_from_default_env().unwrap(),
        None => EnvFilter::new("error"),
    };

    let _ = tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .try_init();

    let mut wallet = WalletUnlocked::load_keystore(
        wallet_path.as_os_str().to_str().unwrap(),
        defaults::WALLET_PASSWORD,
        None,
    )
    .unwrap();

    let number_of_coins = defaults::COIN_AMOUNT;
    let asset_id = AssetId::zeroed();
    let coins = setup_single_asset_coins(
        wallet.address(),
        asset_id,
        number_of_coins,
        DEFAULT_COIN_AMOUNT,
    );

    let addr = match host_str {
        Some(h) => h.parse::<SocketAddr>().unwrap_or_else(|_| {
            derive_socket_addr(defaults::FUEL_NODE_HOST, defaults::FUEL_NODE_PORT)
        }),
        None => derive_socket_addr(defaults::FUEL_NODE_HOST, defaults::FUEL_NODE_PORT),
    };

    let config = Config {
        utxo_validation: false,
        addr,
        ..Config::local_node()
    };

    let (client, _, consensus_params) =
        setup_test_client(coins, vec![], Some(config), None).await;

    let provider = Provider::new(client, consensus_params);

    wallet.set_provider(provider.clone());

    if let Some(contract_bin_path) = contract_bin_path {
        let loaded_contract = Contract::load_from(
            contract_bin_path.as_os_str().to_str().unwrap(),
            LoadConfiguration::default(),
        )
        .expect("Failed to load contract");

        let contract_id = loaded_contract
            .deploy(&wallet, TxParameters::default())
            .await
            .unwrap();

        let contract_id = contract_id.to_string();

        println!("Contract deployed at: {}", &contract_id);
    }

    Ok(())
}

pub async fn setup_example_test_fuel_node() -> Result<(), ()> {
    let wallet_path = Path::new(WORKSPACE_ROOT).join("test-chain-config.json");

    let contract_bin_path = Path::new(WORKSPACE_ROOT)
        .join("contracts")
        .join("fuel-indexer-test")
        .join("out")
        .join("debug")
        .join("fuel-indexer-test.bin");

    setup_test_fuel_node(wallet_path, Some(contract_bin_path), None).await
}

pub fn get_test_contract_id() -> Bech32ContractId {
    let contract_bin_path = Path::new(WORKSPACE_ROOT)
        .join("contracts")
        .join("fuel-indexer-test")
        .join("out")
        .join("debug")
        .join("fuel-indexer-test.bin");

    let loaded_contract = Contract::load_from(
        contract_bin_path.as_os_str().to_str().unwrap(),
        LoadConfiguration::default(),
    )
    .unwrap();
    let id = loaded_contract.contract_id();

    Bech32ContractId::from(fuels::types::ContractId::from(<[u8; 32]>::from(id)))
}

pub async fn api_server_app_postgres(
    database_url: Option<&str>,
    config: Option<IndexerConfig>,
) -> (Router, Receiver<ServiceRequest>) {
    let mut config = config.unwrap_or_default();
    if let Some(url) = database_url {
        config.database = DatabaseConfig::from_str(url).unwrap();
    }

    let pool = IndexerConnectionPool::connect(&config.database.to_string())
        .await
        .unwrap();

    let (tx, rx) = channel::<ServiceRequest>(SERVICE_REQUEST_CHANNEL_SIZE);

    let router = WebApi::build(config, pool, tx).await.unwrap();

    // NOTE: Keep Receiver in scope to prevent the channel from being closed
    (router, rx)
}

pub async fn indexer_service_postgres(
    database_url: Option<&str>,
    config: Option<IndexerConfig>,
) -> IndexerService {
    let mut config = config.unwrap_or_default();
    if let Some(url) = database_url {
        config.database = DatabaseConfig::from_str(url).unwrap();
    }

    let (_tx, rx) = channel::<ServiceRequest>(SERVICE_REQUEST_CHANNEL_SIZE);

    let pool = IndexerConnectionPool::connect(&config.database.to_string())
        .await
        .unwrap();

    IndexerService::new(config, pool, rx).await.unwrap()
}

pub async fn connect_to_deployed_contract(
) -> Result<FuelIndexerTest<WalletUnlocked>, Box<dyn std::error::Error>> {
    let wallet_path = Path::new(WORKSPACE_ROOT).join("test-chain-config.json");
    let wallet_path_str = wallet_path.as_os_str().to_str().unwrap();
    let mut wallet =
        WalletUnlocked::load_keystore(wallet_path_str, defaults::WALLET_PASSWORD, None)
            .unwrap();

    let provider = Provider::connect(defaults::FUEL_NODE_ADDR).await.unwrap();

    wallet.set_provider(provider);

    println!(
        "Wallet({}) keystore at: {}",
        wallet.address(),
        wallet_path.display()
    );

    let contract_id: Bech32ContractId =
        Bech32ContractId::from(fuels::types::ContractId::from(get_test_contract_id()));

    let contract = FuelIndexerTest::new(contract_id.clone(), wallet);

    println!("Using contract at {contract_id}");

    Ok(contract)
}

pub mod test_web {
    use crate::{defaults, fixtures::get_test_contract_id};
    use actix_service::ServiceFactory;
    use actix_web::{
        body::MessageBody,
        dev::{ServiceRequest, ServiceResponse},
        web, App, Error, HttpResponse, HttpServer, Responder,
    };
    use async_std::sync::Arc;
    use fuels::prelude::{CallParameters, Provider, WalletUnlocked};
    use fuels::types::bech32::Bech32ContractId;
    use std::path::Path;

    use super::{tx_params, FuelIndexerTest};

    async fn fuel_indexer_test_blocks(state: web::Data<Arc<AppState>>) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_ping()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_ping(state: web::Data<Arc<AppState>>) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_ping()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_transfer(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let call_params =
            CallParameters::new(1_000_000, fuels::types::AssetId::default(), 1000);

        let _ = state
            .contract
            .methods()
            .trigger_transfer()
            .tx_params(tx_params())
            .call_params(call_params)
            .expect("Could not set call parameters for contract method")
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_log(state: web::Data<Arc<AppState>>) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_log()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_logdata(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_logdata()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_scriptresult(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_scriptresult()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    // TODO: TransferOut is  currently ignored due to flakiness;
    // transfering to an address fails for some unknown reason
    async fn fuel_indexer_test_transferout(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let call_params =
            CallParameters::new(1_000_000, fuels::types::AssetId::default(), 1000);

        let _ = state
            .contract
            .methods()
            .trigger_transferout()
            .tx_params(tx_params())
            .call_params(call_params)
            .expect("Could not set call parameters for contract method")
            .call()
            .await;

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_messageout(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let call_params =
            CallParameters::new(1_000_000, fuels::types::AssetId::default(), 1000);

        let _ = state
            .contract
            .methods()
            .trigger_messageout()
            .call_params(call_params)
            .expect("Could not set call parameters for contract method")
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_callreturn(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_callreturn()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_multiargs(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_multiargs()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_optional_schema_fields(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_ping_for_optional()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_deeply_nested_schema_fields(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_deeply_nested()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_nested_query_explicit_foreign_keys_schema_fields(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_explicit()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();
        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_get_block_height() -> impl Responder {
        let provider = Provider::connect(defaults::FUEL_NODE_ADDR).await.unwrap();
        let block_height = provider.latest_block_height().await.unwrap();

        HttpResponse::Ok().body(block_height.to_string())
    }

    async fn fuel_indexer_test_tuple(state: web::Data<Arc<AppState>>) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_tuple()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();
        HttpResponse::Ok()
    }

    async fn fuel_indexer_vec_calldata(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_vec_pong_calldata(vec![1, 2, 3, 4, 5, 6, 7, 8])
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();
        HttpResponse::Ok()
    }

    async fn fuel_indexer_vec_logdata(state: web::Data<Arc<AppState>>) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_vec_pong_logdata()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();
        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_pure_function(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_pure_function()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_trigger_panic(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_panic()
            .tx_params(tx_params())
            .call()
            .await;

        HttpResponse::Ok()
    }
    async fn fuel_indexer_test_trigger_revert(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_revert()
            .tx_params(tx_params())
            .call()
            .await;

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_trigger_enum_error(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_enum_error(69)
            .tx_params(tx_params())
            .call()
            .await;

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_trigger_enum(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_enum()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_trigger_mint(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_mint()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_trigger_burn(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let call_params =
            CallParameters::new(1_000_000, fuels::types::AssetId::default(), 1000);
        let _ = state
            .contract
            .methods()
            .trigger_burn()
            .tx_params(tx_params())
            .call_params(call_params)
            .unwrap()
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_trigger_generics(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_generics()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    pub struct AppState {
        pub contract: FuelIndexerTest<WalletUnlocked>,
    }

    pub fn app(
        contract: FuelIndexerTest<WalletUnlocked>,
    ) -> App<
        impl ServiceFactory<
            ServiceRequest,
            Response = ServiceResponse<impl MessageBody>,
            Config = (),
            InitError = (),
            Error = Error,
        >,
    > {
        let state = web::Data::new(Arc::new(AppState { contract }));
        App::new()
            .app_data(state)
            .route("/block", web::post().to(fuel_indexer_test_blocks))
            .route("/ping", web::post().to(fuel_indexer_test_ping))
            .route("/transfer", web::post().to(fuel_indexer_test_transfer))
            .route("/log", web::post().to(fuel_indexer_test_log))
            .route("/logdata", web::post().to(fuel_indexer_test_logdata))
            .route(
                "/scriptresult",
                web::post().to(fuel_indexer_test_scriptresult),
            )
            .route(
                "/transferout",
                web::post().to(fuel_indexer_test_transferout),
            )
            .route("/messageout", web::post().to(fuel_indexer_test_messageout))
            .route("/returndata", web::post().to(fuel_indexer_test_callreturn))
            .route("/multiarg", web::post().to(fuel_indexer_test_multiargs))
            .route(
                "/optionals",
                web::post().to(fuel_indexer_test_optional_schema_fields),
            )
            .route(
                "/block_height",
                web::get().to(fuel_indexer_test_get_block_height),
            )
            .route("/tuples", web::post().to(fuel_indexer_test_tuple))
            .route(
                "/deeply_nested",
                web::post().to(fuel_indexer_test_deeply_nested_schema_fields),
            )
            .route(
                "/explicit",
                web::post().to(
                    fuel_indexer_test_nested_query_explicit_foreign_keys_schema_fields,
                ),
            )
            .route("/vec_calldata", web::post().to(fuel_indexer_vec_calldata))
            .route("/vec_logdata", web::post().to(fuel_indexer_vec_logdata))
            .route("/call", web::post().to(fuel_indexer_test_pure_function))
            .route("/panic", web::post().to(fuel_indexer_test_trigger_panic))
            .route("/revert", web::post().to(fuel_indexer_test_trigger_revert))
            .route(
                "/enum_error",
                web::post().to(fuel_indexer_test_trigger_enum_error),
            )
            .route("/enum", web::post().to(fuel_indexer_test_trigger_enum))
            .route("/mint", web::post().to(fuel_indexer_test_trigger_mint))
            .route("/burn", web::post().to(fuel_indexer_test_trigger_burn))
            .route(
                "/generics",
                web::post().to(fuel_indexer_test_trigger_generics),
            )
    }

    pub async fn server() -> Result<(), Box<dyn std::error::Error>> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;

        let wallet_path = Path::new(&manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("test-chain-config.json");

        let wallet_path_str = wallet_path.as_os_str().to_str().unwrap();
        let mut wallet = WalletUnlocked::load_keystore(
            wallet_path_str,
            defaults::WALLET_PASSWORD,
            None,
        )
        .unwrap();

        let provider = Provider::connect(defaults::FUEL_NODE_ADDR).await.unwrap();

        wallet.set_provider(provider.clone());

        println!(
            "Wallet({}) keystore at: {}",
            wallet.address(),
            wallet_path.display()
        );

        let contract_id: Bech32ContractId = get_test_contract_id();

        println!("Starting server at {}", defaults::WEB_API_ADDR);

        let _ = HttpServer::new(move || {
            app(FuelIndexerTest::new(contract_id.clone(), wallet.clone()))
        })
        .bind(defaults::WEB_API_ADDR)
        .unwrap()
        .run()
        .await;

        Ok(())
    }
}
