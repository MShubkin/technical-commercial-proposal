use asez2_shared_db::{
    db_item::{AsezTimestamp, Select},
    uuid,
};
use uuid::Uuid;

use super::super::*;
use crate::application::background::request_header_closing::request_header_closing;

const MIGS: &[&str] = &["background/request_header_closing.sql"];
const TECH_USER: i32 = 123;

#[tokio::test]
async fn success() {
    run_db_test(MIGS, |pool| async move {
        let now = AsezTimestamp::now();

        request_header_closing(pool.clone(), TECH_USER).await.unwrap();

        let headers = RequestHeader::select(
            &Select::full::<RequestHeader>()
                .add_replace_order_asc(RequestHeader::plan_id),
            pool.as_ref(),
        )
        .await
        .unwrap();
        println!(
            "{}",
            AsezTimestamp::try_from_db_format("2020-01-01 10:10:10").unwrap() < now
        );
        let verify_header = |header_uuid: Uuid, is_updated: bool| {
            let header =
                headers.iter().find(|h| h.uuid == header_uuid).unwrap_or_else(
                    || panic!("Не найден заголовок с uuid={header_uuid}"),
                );

            if is_updated {
                assert!(header.changed_at > now);
                assert_eq!(header.changed_by, TECH_USER);
                assert_eq!(
                    header.status_id,
                    PriceInformationRequestStatus::EntryClosed
                );
            } else {
                assert!(
                    header.changed_at < now,
                    "Запись {header_uuid} не должна была обновиться"
                )
            }
        };

        [
            (uuid!("00000000-0000-0000-0000-000000000001"), false),
            (uuid!("00000000-0000-0000-0000-000000000002"), false),
            (uuid!("00000000-0000-0000-0000-000000000003"), true),
        ]
        .into_iter()
        .for_each(|(h, is_updated)| verify_header(h, is_updated));
    })
    .await
}
