mod actor;
mod handle;

pub use handle::Debounce;

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use rstest::*;

    use super::*;

    #[fixture]
    pub fn fixture() -> Debounce<i32> {
        Debounce::new(Duration::seconds(1))
    }

    #[rstest]
    #[tokio::test]
    #[timeout(std::time::Duration::from_millis(100))]
    async fn returns_first_immediately(mut fixture: Debounce<i32>) {
        fixture.set(42).await;

        let start = std::time::Instant::now();
        assert_eq!(fixture.get().await, 42);
        assert_eq!(start.elapsed().as_secs(), 0);
    }

    #[rstest]
    #[tokio::test]
    async fn first_get_always_without_delay_take_last_value_1(mut fixture: Debounce<i32>) {
        let start = std::time::Instant::now();
        fixture.set(1).await;
        fixture.set(2).await;

        assert_eq!(fixture.get().await, 2);
        assert_eq!(start.elapsed().as_secs(), 0);
    }

    #[rstest]
    #[tokio::test]
    async fn first_get_always_without_delay_take_last_value_2(mut fixture: Debounce<i32>) {
        let start = std::time::Instant::now();
        fixture.set(1).await;
        fixture.set(2).await;
        fixture.set(3).await;

        assert_eq!(fixture.get().await, 3);
        assert_eq!(start.elapsed().as_secs(), 0);
    }

    #[rstest]
    #[tokio::test]
    async fn second_get_has_delay(mut fixture: Debounce<i32>) {
        fixture.set(1).await;
        fixture.set(2).await;
        fixture.set(3).await;

        let start = std::time::Instant::now();
        assert_eq!(fixture.get().await, 3);
        assert_eq!(start.elapsed().as_secs(), 0);

        let start = std::time::Instant::now();
        fixture.set(4).await;
        assert_eq!(fixture.get().await, 4);
        assert_eq!(start.elapsed().as_secs(), 1);
    }

    #[rstest]
    #[tokio::test]
    async fn second_get_has_delay_even_when_not_finished(mut fixture: Debounce<i32>) {
        fixture.set(1).await;
        fixture.set(2).await;
        fixture.set(3).await;

        assert_eq!(fixture.get().await, 3);

        let start = std::time::Instant::now();
        fixture.set(4).await;
        fixture.set(5).await;
        assert_eq!(fixture.get().await, 5);
        assert_eq!(start.elapsed().as_secs(), 1);
    }

    #[rstest]
    #[tokio::test]
    async fn sleeping_will_settle_delay(mut fixture: Debounce<i32>) {
        fixture.set(1).await;
        fixture.set(2).await;
        fixture.set(3).await;

        assert_eq!(fixture.get().await, 3);

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        let start = std::time::Instant::now();
        fixture.set(4).await;
        assert_eq!(fixture.get().await, 4);
        assert_eq!(start.elapsed().as_secs(), 0);
    }

    #[rstest]
    #[tokio::test]
    async fn sleeping_will_settle_delay_2(mut fixture: Debounce<i32>) {
        fixture.set(1).await;
        fixture.set(2).await;
        fixture.set(3).await;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let start = std::time::Instant::now();
        assert_eq!(fixture.get().await, 3);
        assert_eq!(start.elapsed().as_secs(), 0);
    }

    #[rstest]
    #[tokio::test]
    async fn sleeping_will_settle_delay_3(mut fixture: Debounce<i32>) {
        fixture.set(1).await;
        fixture.set(2).await;
        fixture.set(3).await;

        assert_eq!(fixture.get().await, 3);

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        let start = std::time::Instant::now();
        fixture.set(4).await;
        fixture.set(5).await;
        fixture.set(6).await;
        assert_eq!(fixture.get().await, 6);
        assert_eq!(start.elapsed().as_secs(), 0);
    }
}
