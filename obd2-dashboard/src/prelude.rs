use embassy_time::Duration;

pub trait FutureTimeout {
    type Output;
    async fn timeout(self, dur: embassy_time::Duration) -> Result<Self::Output, embassy_time::TimeoutError>;
    async fn timeout_secs(self, secs: u64) -> Result<Self::Output, embassy_time::TimeoutError>;
    async fn timeout_millis(self, millis: u64) -> Result<Self::Output, embassy_time::TimeoutError>;
}

impl<F> FutureTimeout for F
where
    F: core::future::Future,
{
    type Output = F::Output;
    async fn timeout(self, dur: embassy_time::Duration) -> Result<Self::Output, embassy_time::TimeoutError> {
        embassy_time::with_timeout(dur, self).await
    }
    async fn timeout_secs(self, secs: u64) -> Result<Self::Output, embassy_time::TimeoutError> {
        self.timeout(Duration::from_secs(secs)).await
    }
    async fn timeout_millis(self, millis: u64) -> Result<Self::Output, embassy_time::TimeoutError> {
        self.timeout(Duration::from_millis(millis)).await
    }
}
