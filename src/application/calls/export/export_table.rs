use crate::{
    application::calls::{
        database_request, get_price_information_request_info_details_export,
    },
    presentation::dto::PriceInformationDetail,
};
use asez2_shared_db::db_item::Select;
use rabbit_services::print_doc::PrintDocService;
use shared_essential::{
    common::export::{build_export_table, FeWrapperRepLookup, FieldLookup},
    domain::{
        tcp::{
            PriceInformationRequestStatus, RequestHeader, RequestHeaderRep,
            RequestItem, RequestItemRep,
        },
        technical_commercial_proposal::request_type::PriceInformationRequestTypeId,
    },
    presentation::dto::{
        export::default_replacement_config,
        general::{
            DataRecord, DataRecords, ExportResponse, FeWrapper, InternalExportReq,
            TaggedValue, UiExportTableReq, UiSelect,
        },
        print_docs::{common::TemplateFormat, Content, PrintReq},
        response_request::{EntityKind, MessageKind, Messages},
        technical_commercial_proposal::{TcpError, TcpResult, UiSection},
    },
    replacement,
};
use sqlx::PgPool;
use tracing::info;

pub async fn process_export_table(
    request: UiExportTableReq<UiSection>,
    user_id: i32,
    monolith_token: String,
    pool: &PgPool,
    print_doc: &PrintDocService,
) -> TcpResult<(ExportResponse, Messages)> {
    info!(kind = "tcp", "Export table request {:?}", request);

    let UiExportTableReq {
        section_id,
        format,
        select,
        captions,
        ..
    } = request;

    let mut messages = Messages::default();
    let (entity_kind, data) = match section_id {
        UiSection::RequestList => {
            let data =
                get_request_headers_data_records(select.clone(), pool).await?;
            (vec![EntityKind::RequestList; data.len()], data)
        }
        UiSection::RequestItem => {
            let data = get_request_items_data_records(select.clone(), pool).await?;
            (vec![EntityKind::RequestItem; data.len()], data)
        }
        other => {
            return Err(TcpError::Section(format!(
                "Секция {other} не имеет возможности экспорта таблицы"
            )))
        }
    };
    if data.is_empty() {
        messages.add_message(
            MessageKind::Error,
            "По данному запросу данных не найдено".to_owned(),
        );
        return Err(TcpError::Business(messages));
    }

    let replacements = default_replacement_config()
        .chain([
            replacement!(okved2_id: planning_dict(Okved2) as Code),
            replacement!(okpd2_id: planning_dict(Okpd2) as Code),
            replacement!(product_type_id: nsi_dict(PpzType) as Text),
            replacement!(organizer_id: planning_common_dict(Customer) as Text),
            replacement!(
                purchasing_trend_id: planning_common_dict(PurchasingTrend) as Text
            ),
            replacement!(
                type_request_id: enum_display(PriceInformationRequestTypeId)
            ),
            replacement!(status_id: enum_display(PriceInformationRequestStatus)),
            replacement!(supplier_list: Length),
            replacement!(tkp_in: Length),
            replacement!(tkp_done: joined_list("/")),
        ])
        .collect();

    let content = Content {
        extension: format.unwrap_or(TemplateFormat::Xlsx),
        confidentially: false,
        input_content: PrintReq::XlsxExport(InternalExportReq {
            format,
            user_id,
            replacements,
            monolith_token,
            template: Some("export_table".to_owned()),
            data: DataRecords {
                data,
                entity_kind,
                field_list: select.field_list,
                captions: captions.unwrap_or_default(),
            },
        }),
    };
    let document = print_doc.create_document(&content).await.map_err(|e| {
        TcpError::Export(format!(
            "Ошибка формирования документа из сервиса print-doc: {e}"
        ))
    })?;
    let byte_buf = match document.buf {
        None => {
            return Err(TcpError::Export(
                "Отсутствует содержимое документа".to_owned(),
            ));
        }
        Some(buf) => buf,
    };
    Ok((ExportResponse { byte_buf }, messages))
}

async fn get_request_headers_data_records(
    select: UiSelect,
    pool: &PgPool,
) -> TcpResult<Vec<DataRecord>> {
    let db_select = select.clone().try_into()?;
    let data = database_request(db_select, pool).await?;

    Ok(build_export_table::<RequestHeaderLookup>(
        data,
        select.field_list.iter().map(AsRef::as_ref),
    ))
}

async fn get_request_items_data_records(
    select: UiSelect,
    pool: &PgPool,
) -> TcpResult<Vec<DataRecord>> {
    let db_select: Select = select.clone().try_into()?;

    let (item_select, header_select) = db_select.split_for::<RequestItemRep>();
    let item_select = item_select.add_replace_order_asc(RequestItem::number);

    let joined_details = get_price_information_request_info_details_export(
        header_select,
        item_select,
        pool,
    )
    .await?;

    let details = PriceInformationDetail::from(joined_details);
    let plan_id = details.request_header.plan_id.flatten();
    let currency_id = details.request_header.currency_id.flatten();

    Ok(build_export_table::<RequestItemLookup>(
        details.item_list.into_iter().map(|item| (plan_id, currency_id, item)),
        select.field_list.iter().map(AsRef::as_ref),
    ))
}

type RequestHeaderLookup = FeWrapperRepLookup<RequestHeaderRep>;

struct RequestItemLookup {
    plan_id: Option<i64>,
    currency_id: Option<i16>,
    inner: FeWrapperRepLookup<RequestItemRep>,
}

impl<'fields> FieldLookup<'fields> for RequestItemLookup {
    type Source = (Option<i64>, Option<i16>, FeWrapper<RequestItemRep>);
    type Field = &'fields str;

    fn build((plan_id, currency_id, inner): Self::Source) -> Self {
        Self {
            plan_id,
            currency_id,
            inner: FeWrapperRepLookup::build(inner),
        }
    }

    fn get_or_null(&self, pos: usize, field: Self::Field) -> TaggedValue {
        match field {
            RequestHeader::plan_id => self.plan_id.into(),
            RequestHeader::currency_id => self.currency_id.into(),
            other => self.inner.get_or_null(pos, other),
        }
    }
}
