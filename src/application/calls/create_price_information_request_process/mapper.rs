use crate::presentation::dto::PriceInformationRequest;
use asez2_shared_db::db_item::AsezTimestamp;
use shared_essential::domain::tables::tcp::PriceInformationRequestType;
use shared_essential::domain::tables::tcp::{
    RequestHeader, RequestItem, RequestPartner,
};
use shared_essential::presentation::dto::technical_commercial_proposal::{
    create_price_information_request::{CreatePriceInformationRequest, PlanUUIDs},
    TcpError,
};
use uuid::Uuid;

impl PriceInformationRequest {
    /// Первоначальное заполнение стуктуры запроса ценовой информации на основе пользовательских данных
    pub(crate) fn create_price_information_request_from_json(
        json_dto: CreatePriceInformationRequest,
    ) -> Result<Vec<Self>, TcpError> {
        // Список создаваемызх ЗЦИ
        let mut request_list: Vec<Self> =
            Vec::with_capacity(json_dto.plan_data.len());

        // UUID ППЗ и позиций
        let plan_data_json_list = &json_dto.plan_data;

        for plan_data_json in plan_data_json_list {
            let header = Self::create_header(&plan_data_json.plan_uuid, &json_dto)?;
            let items = Self::create_items(header.uuid, plan_data_json)?;
            let suppliers = Self::create_suppliers(header.uuid, &json_dto);

            request_list.push(Self {
                header,
                items,
                suppliers,
            });
        }

        Ok(request_list)
    }

    /// Заполнение заголовка ЗЦИ
    fn create_header(
        plan_uuid: &str,
        json_dto: &CreatePriceInformationRequest,
    ) -> Result<RequestHeader, TcpError> {
        Ok(RequestHeader {
            uuid: Uuid::new_v4(),
            plan_uuid: Some(Uuid::parse_str(plan_uuid)?),
            end_date: Some(json_dto.period_of_validity),
            type_request_id: Some(PriceInformationRequestType::from(json_dto.request_type)),
            status_id:
                shared_essential::domain::tables::tcp::PriceInformationRequestStatus::Created,
            created_at: AsezTimestamp::now(),
            ..Default::default()
        })
    }

    /// Заполнение позиций
    fn create_items(
        header_uuid: Uuid,
        plan_uuids: &PlanUUIDs,
    ) -> Result<Vec<RequestItem>, TcpError> {
        let mut items: Vec<RequestItem> =
            Vec::with_capacity(plan_uuids.plan_item_uuids.len());
        let json_plan_item_uuis = &plan_uuids.plan_item_uuids;
        for json_plan_item_uui in json_plan_item_uuis {
            let item = RequestItem {
                uuid: Uuid::new_v4(),
                request_uuid: header_uuid,
                plan_item_uuid: Uuid::parse_str(json_plan_item_uui.as_str())?,
                ..Default::default()
            };
            items.push(item);
        }
        Ok(items)
    }

    /// Заполнение струткуры с поставщиками
    fn create_suppliers(
        header_uuid: Uuid,
        json_dto: &CreatePriceInformationRequest,
    ) -> Option<Vec<RequestPartner>> {
        if let Some(json_suppliers) = &json_dto.suppliers {
            let mut suppliers: Vec<RequestPartner> =
                Vec::with_capacity(json_suppliers.len());
            for json_supplier in json_suppliers {
                let supplier = RequestPartner {
                    uuid: Uuid::new_v4(),
                    request_uuid: header_uuid,
                    supplier_id: json_supplier.supplier_id,
                    ..Default::default()
                };
                suppliers.push(supplier);
            }
            Some(suppliers)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::presentation::dto::PriceInformationRequest;
    use shared_essential::presentation::dto::technical_commercial_proposal::create_price_information_request::{CreatePriceInformationRequest, FileFormData, PlanUUIDs, SupplierFormData};

    #[test]
    fn test_create_price_information_request_from_json() {
        let plan1 = PlanUUIDs {
            plan_uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            plan_item_uuids: vec![
                "550e8400-e29b-41d4-a716-446655440001".to_string(),
                "550e8400-e29b-41d4-a716-446655440002".to_string(),
            ],
        };

        let plan2 = PlanUUIDs {
            plan_uuid: "550e8400-e29b-41d4-a716-446655441000".to_string(),
            plan_item_uuids: vec![
                "550e8400-e29b-41d4-a716-146655440001".to_string(),
                "510e8400-e29b-41d4-a716-446655440002".to_string(),
            ],
        };

        let supplier1 = SupplierFormData {
            supplier_id: 1111,
            additional_email: Some("email1".to_string()),
        };
        let supplier2 = SupplierFormData {
            supplier_id: 2222,
            additional_email: Some("email2".to_string()),
        };

        let technical_specification = FileFormData {
            uuid: "550e8400-e29b-41d4-a717-446655441000".to_string(),
            name: "technical_specification.doc".to_string(),
        };

        let draft_treaty = FileFormData {
            uuid: "550e8400-e29b-41d4-a717-446655441100".to_string(),
            name: "draft_treaty.doc".to_string(),
        };

        let template_tkp = FileFormData {
            uuid: "510e8400-e39b-41d4-a716-446655440002".to_string(),
            name: "template_tkp.doc".to_string(),
        };

        let additional_document1 = FileFormData {
            uuid: "510e8400-e29b-41d4-a717-446655440002".to_string(),
            name: "additional_document1.doc".to_string(),
        };

        let additional_document2 = FileFormData {
            uuid: "510e8500-e29b-41d4-a716-446655440002".to_string(),
            name: "additional_document2.doc".to_string(),
        };

        let crate_json = CreatePriceInformationRequest {
            plan_data: vec![plan1, plan2],
            period_of_validity: Default::default(),
            request_type: 1,
            suppliers: Some(vec![supplier1, supplier2]),
            technical_specification,
            draft_treaty,
            template_tkp,
            additional_documents: Some(vec![
                additional_document1,
                additional_document2,
            ]),
        };

        let requests =
            PriceInformationRequest::create_price_information_request_from_json(
                crate_json,
            )
            .unwrap();
        assert_eq!(requests.len(), 2_usize);
        for request in requests {
            assert_eq!(request.items.len(), 2_usize);
            assert_eq!(request.suppliers.unwrap().len(), 2_usize);
        }
    }
}
