mod background;

mod add_partner;
mod apply_proposal_pricing;
mod approve_proposal;
mod delete_price_info;
mod get_price_details;
mod get_proposal_detail;
mod get_proposal_items_for_pricing;
mod get_proposals_by_object_id;
mod get_request_price_info_list;
mod organization_question;
mod price_info_close;
mod price_info_complete;
mod update_price_info;
mod update_proposal;

mod get_technical_commercial_proposal;

use asez2_shared_db::DbItem;
use env_setup::MonolithCfg;
use monolith_service::http::{MonolithHttpDriver, MonolithHttpService};
use monolith_service::MonolithService;
use shared_essential::domain::tables::tcp::*;

use sqlx::PgPool;
use std::sync::Arc;

pub(crate) const TCP_TRANSIENT_TABLES: &[&str] = &[
    OrganizationQuestion::TABLE,
    ProposalHeader::TABLE,
    ProposalItem::TABLE,
    RequestHeader::TABLE,
    RequestItem::TABLE,
    RequestPartner::TABLE,
    StatusHistory::TABLE,
];

pub(crate) async fn run_db_test<F, FutFn>(
    extra_migs: &'static [&'static str],
    run: FutFn,
) where
    F: futures::Future<Output = ()>,
    FutFn: FnOnce(Arc<PgPool>) -> F + 'static,
{
    testing::BaseMigPath::MigrationsHome
        .run_test_with_migrations(
            "src/application/tests/extra_migrations", // Extra migs dir
            extra_migs,                               // Extra migs
            TCP_TRANSIENT_TABLES,
            run,
        )
        .await
}

pub(crate) async fn run_db_test_with_monolith<F, FutFn>(
    extra_migs_files: &'static [&'static str],
    run: FutFn,
) where
    F: futures::Future<Output = ()>,
    FutFn: FnOnce(Arc<PgPool>, MonolithHttpService) -> F + 'static,
{
    let monolith_cfg =
        MonolithCfg::from_env().expect("Ошибка при чтении конфига монолита");
    let monolith_driver = MonolithHttpDriver::basic_driver(monolith_cfg.url)
        .expect("Ошибка при настройке http драйвера монолита");

    run_db_test(extra_migs_files, |pool| async move {
        run(pool, MonolithService::new(monolith_driver)).await
    })
    .await
}
mod check_request_price_info;
mod delete_partner;
