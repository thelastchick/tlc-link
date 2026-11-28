use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("هسته {0} پیدا نشد. فایل اجرایی آن را در پوشه binaries قرار دهید.")]
    CoreMissing(String),
    #[error("اجرای هسته ناموفق بود: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("خطای کانفیگ: {0}")]
    InvalidConfig(String),
    #[error("هیچ اتصالی فعال نیست")]
    NotConnected,
    #[error("فرایند هسته هنوز فعال است و متوقف نشد")]
    StopFailed,
}
