use std::{
    cell::OnceCell,
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use config::Config;
use rustls_pemfile::certs;
use thiserror::Error;
use tokio_rustls::rustls::{self, Certificate, PrivateKey};

fn load_certs<P: AsRef<Path>>(path: P) -> io::Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
        .map(|certs| certs.into_iter().map(Certificate).collect())
}

fn load_keys<P: AsRef<Path>>(path: P) -> io::Result<Vec<PrivateKey>> {
    rustls_pemfile::read_all(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
        .map(|keys| {
            keys.into_iter()
                .filter_map(|x| match x {
                    // Item::X509Certificate(cert) => println!("certificate {:?}", cert),
                    rustls_pemfile::Item::RSAKey(key)
                    | rustls_pemfile::Item::PKCS8Key(key)
                    | rustls_pemfile::Item::ECKey(key) => Some(PrivateKey(key)),
                    _ => None,
                })
                .collect()
        })
}

pub trait OnceCellExt<T> {
    fn get_or_init_fallible<E, F: FnOnce() -> Result<T, E>>(&self, f: F) -> Result<&T, E>;
}

impl<T> OnceCellExt<T> for OnceCell<T> {
    fn get_or_init_fallible<E, F: FnOnce() -> Result<T, E>>(&self, f: F) -> Result<&T, E> {
        if let Some(r) = self.get() {
            Ok(r)
        } else {
            let val = f()?;
            Ok(self.get_or_init(|| val))
        }
    }
}

#[derive(Default)]
pub struct CertificatesCache {
    ca: OnceCell<Vec<Certificate>>,
    own_cert: OnceCell<Vec<Certificate>>,
    own_key: OnceCell<PrivateKey>,
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Rustls(#[from] rustls::Error),
    #[error("No key in file")]
    NoKey,
}

impl CertificatesCache {
    pub fn get_ca(&self, conf: &Config) -> Result<&[Certificate], CacheError> {
        self.ca
            .get_or_init_fallible(|| load_certs(&conf.node.ca_file).map_err(Into::into))
            .map(|v| v.as_slice())
    }

    pub fn get_cert(&self, conf: &Config) -> Result<Vec<Certificate>, CacheError> {
        self.own_cert
            .get_or_init_fallible(|| load_certs(&conf.node.cert_file).map_err(Into::into))
            .map(Clone::clone)
    }

    pub fn get_key(&self, conf: &Config) -> Result<PrivateKey, CacheError> {
        self.own_key
            .get_or_init_fallible(|| {
                load_keys(&conf.node.key_file)
                    .map_err(Into::into)
                    .and_then(|mut keys| {
                        if keys.is_empty() {
                            Err(CacheError::NoKey)
                        } else {
                            Ok(keys.remove(0))
                        }
                    })
            })
            .map(Clone::clone)
    }
}
