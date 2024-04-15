use defmt::unwrap;

use crate::types::Sh1122;

#[embassy_executor::task]
pub async fn run(mut display1: Sh1122<10>, mut display2: Sh1122<1>) {
    unwrap!(display1.init(None).await);
    unwrap!(display2.init(None).await);

    display1.clear();
    display2.clear();

    unwrap!(display1.flush().await);
    unwrap!(display2.flush().await);
}
