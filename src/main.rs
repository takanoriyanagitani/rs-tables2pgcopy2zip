use std::process::ExitCode;

use std::io;

use std::io::BufRead;
use std::io::Write;

use std::fs::File;

use zip::ZipWriter;

use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};

use tokio_postgres::Config;
use tokio_postgres::NoTls;

use rs_tables2pgcopy2zip::tablename2pgcopy2zip;

const POOL_SIZE: usize = 2;

async fn tablenames2pgcopy2zip<I>(
    zipfilename: String,
    trusted_tablenames: I,
) -> Result<(), io::Error>
where
    I: Iterator<Item = String>,
{
    let mut pcfg: Config = Config::new();

    let host: String = env2pghost()?;
    let user: String = env2pguser()?;
    let pass: String = env2pgpass()?;
    let db: String = env2pgdb()?;

    pcfg.host(host);
    pcfg.user(user);
    pcfg.dbname(db);
    pcfg.password(pass);

    let mcfg = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let mgr: Manager = Manager::from_config(pcfg, NoTls, mcfg);
    let pool: Pool = Pool::builder(mgr)
        .max_size(POOL_SIZE)
        .build()
        .map_err(io::Error::other)?;

    let client = pool.get().await.map_err(io::Error::other)?;

    let mut zfile: File = File::create(zipfilename)?;
    let mut zw: ZipWriter<_> = ZipWriter::new(&mut zfile);

    for tname in trusted_tablenames {
        tablename2pgcopy2zip(&tname, &mut zw, &client).await?;
    }

    zw.finish()?;

    zfile.flush()?;
    zfile.sync_data()?;

    Ok(())
}

fn env_val_by_key(key: &'static str) -> impl FnMut() -> Result<String, io::Error> {
    move || std::env::var(key).map_err(io::Error::other)
}

fn env2zipname() -> Result<String, io::Error> {
    env_val_by_key("ENV_ZIP_FILENAME")()
}

fn env2pghost() -> Result<String, io::Error> {
    env_val_by_key("PGHOST")()
}

fn env2pguser() -> Result<String, io::Error> {
    env_val_by_key("PGUSER")()
}

fn env2pgpass() -> Result<String, io::Error> {
    env_val_by_key("PGPASSWORD")()
}

fn env2pgdb() -> Result<String, io::Error> {
    env_val_by_key("PGDATABASE")()
}

async fn stdin2trusted_tablenames2pgcopy2zip() -> Result<(), io::Error> {
    let zipname: String = env2zipname()?;

    let i = io::stdin();
    let l = i.lock();
    let lines = l.lines();
    let noerr = lines.map_while(Result::ok);

    tablenames2pgcopy2zip(zipname, noerr).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    stdin2trusted_tablenames2pgcopy2zip()
        .await
        .map(|_| ExitCode::SUCCESS)
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            ExitCode::FAILURE
        })
}
