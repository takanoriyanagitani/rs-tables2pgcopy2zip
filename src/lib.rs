use std::io;

use std::io::Seek;
use std::io::Write;

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use bytes::Bytes;

use tokio_postgres::Client;
use tokio_postgres::CopyOutStream;

use futures_util::stream::StreamExt;
use futures_util::stream::TryStreamExt;

pub async fn tablename2pgcopy2zip<W>(
    trusted_table_name: &str,
    zw: &mut ZipWriter<W>,
    client: &Client,
) -> Result<(), io::Error>
where
    W: Write + Seek,
{
    let filename: String = format!("{trusted_table_name}.pgcopy.dat");
    zw.start_file(filename, SimpleFileOptions::default())?;

    let query: String = format!("COPY {trusted_table_name} TO STDOUT WITH BINARY");

    let strm: CopyOutStream = client.copy_out(&query).await.map_err(io::Error::other)?;

    let mapd = strm.map(|rslt| {
        rslt.map_err(io::Error::other).and_then(|chunk: Bytes| {
            let s: &[u8] = &chunk;
            zw.write_all(s)?;
            Ok(1)
        })
    });

    let _: usize = mapd
        .try_fold(0, |state, next| async move { Ok(state + next) })
        .await?;

    zw.flush()?;
    Ok(())
}
