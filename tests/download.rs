#[cfg(feature = "download")]
mod download {

    use arkiv::{Archive, Downloader, Error as ArkivError};
    use httptest::{matchers::request, responders::status_code, Expectation, Server};
    use std::{
        fs::{self, File},
        io::{BufReader, Read},
        path::Path,
    };

    type Error = Box<dyn std::error::Error>;
    type Result<T> = std::result::Result<T, Error>;

    #[allow(unused)]
    async fn test_404(path: impl AsRef<Path>) -> Result<()> {
        // read archive contents into buffer
        let archive_file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(archive_file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        // prepare test server to return archive contents on request
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path(
                "GET",
                format!("/{}", path.as_ref().display()),
            ))
            .respond_with(status_code(404)),
        );

        // download archive
        let url = format!("/{}", path.as_ref().display());
        let url = server.url(&url);

        let res = Archive::download(url.to_string());
        assert!(matches!(res, Err(ArkivError::InvalidRequest(_))));

        Ok(())
    }

    #[allow(unused)]
    async fn test_progress(path: impl AsRef<Path>) -> Result<()> {
        // read archive contents into buffer
        let archive_file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(archive_file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        // prepare test server to return archive contents on request
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path(
                "GET",
                format!("/{}", path.as_ref().display()),
            ))
            .respond_with(status_code(200).body(buffer)),
        );

        // download archive
        let url = format!("/{}", path.as_ref().display());
        let url = server.url(&url);

        let mut callbacks: Vec<(u64, u64)> = vec![];
        let mut archive = Downloader::new()
            .url(url.to_string())
            .to_temp()
            .on_progress(|current, total| callbacks.push((current, total)))
            .download()?;

        // check callback was called at least at startup and finish
        let total_filesize = fs::metadata(&path).unwrap().len();
        assert!(!callbacks.is_empty());
        assert!(callbacks.first().unwrap() == &(0, total_filesize));
        assert!(callbacks.last().unwrap() == &(total_filesize, total_filesize));

        // check the contents
        let mut actual = archive.entries()?;
        let mut expected = vec!["sample/", "sample/sample.txt"];
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected);

        Ok(())
    }

    #[allow(unused)]
    async fn test(path: impl AsRef<Path>) -> Result<()> {
        // read archive contents into buffer
        let archive_file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(archive_file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        // prepare test server to return archive contents on request
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path(
                "GET",
                format!("/{}", path.as_ref().display()),
            ))
            .respond_with(status_code(200).body(buffer)),
        );

        // download archive
        let url = format!("/{}", path.as_ref().display());
        let url = server.url(&url);
        let mut archive = Archive::download(url.to_string())?;

        // check the contents
        let mut actual = archive.entries()?;
        let mut expected = vec!["sample/", "sample/sample.txt"];
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected);

        Ok(())
    }

    #[tokio::test]
    async fn download_404() -> Result<()> {
        test_404("tests/sample/sample.zip").await
    }

    #[tokio::test]
    #[cfg(feature = "zip")]
    async fn download_progress() -> Result<()> {
        test_progress("tests/sample/sample.zip").await
    }

    #[tokio::test]
    #[cfg(feature = "zip")]
    async fn zip_archive() -> Result<()> {
        test("tests/sample/sample.zip").await
    }

    #[tokio::test]
    #[cfg(all(feature = "gzip", feature = "tar"))]
    async fn tar_gz_archive() -> Result<()> {
        test("tests/sample/sample.tar.gz").await?;
        test("tests/sample/sample.tgz").await
    }

    #[tokio::test]
    #[cfg(all(feature = "bzip", feature = "tar"))]
    async fn tar_bz2_archive() -> Result<()> {
        test("tests/sample/sample.tar.bz2").await
    }

    #[tokio::test]
    #[cfg(all(feature = "xz", feature = "tar"))]
    async fn tar_xz_archive() -> Result<()> {
        test("tests/sample/sample.tar.xz").await
    }

    #[tokio::test]
    #[cfg(all(feature = "zstd", feature = "tar"))]
    async fn tar_zst_archive() -> Result<()> {
        test("tests/sample/sample.tar.zstd").await?;
        test("tests/sample/sample.tar.zst").await
    }
}
