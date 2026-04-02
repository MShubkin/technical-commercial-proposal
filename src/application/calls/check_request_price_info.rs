use crate::presentation::dto::UpdatePriceInformationRequest;
use monolith_service::dto::attachment::FoldersCategory;
use shared_essential::domain::tables::tcp::PriceInformationRequestType;
use shared_essential::presentation::dto::response_request::{Message, Messages};

pub(crate) async fn check_request_price_info(
    request: &UpdatePriceInformationRequest,
) -> Messages {
    let mut messages = Messages::default();

    if request
        .header
        .type_request_id
        .flatten()
        .as_ref()
        .map_or(true, |item| *item as i16 == 0)
    {
        messages.add_prepared_message(
            CheckRequestPriceInfoMessage::FillField(
                "Тип ЗЦИ".to_owned(),
                "type_request_id".to_owned(),
            )
            .create_message(),
        );
    }

    [
        (&request.header.request_subject, "Предмет ЗЦИ", "request_subject"),
        (&request.header.organizer_name, "Контактное лицо", "organizer_name"),
        (&request.header.organizer_mail, "Электронный адрес", "organizer_mail"),
        (&request.header.organizer_phone, "Телефон", "organizer_phone"),
        (
            &request.header.organizer_location,
            "Местонахождение",
            "organizer_location",
        ),
    ]
    .into_iter()
    .for_each(|(field_id, field_code, field)| {
        if field_id.clone().flatten().map_or(true, |item| item.is_empty()) {
            messages.add_prepared_message(
                CheckRequestPriceInfoMessage::FillField(
                    (*field_code).to_owned(),
                    field.to_owned(),
                )
                .create_message(),
            );
        }
    });

    if matches!(
        request.header.type_request_id,
        Some(Some(PriceInformationRequestType::Private))
    ) {
        if request
            .header
            .request_type_text
            .as_ref()
            .and_then(|x| x.as_ref())
            .map_or(true, |text| text.is_empty())
        {
            messages.add_prepared_message(
                CheckRequestPriceInfoMessage::CloseRequestReason.create_message(),
            );
        }

        if request.partner_list.is_empty() {
            messages.add_prepared_message(
                CheckRequestPriceInfoMessage::Fill(String::from(
                    "данные организаций",
                ))
                .create_message(),
            );
        }
    }

    [
        FoldersCategory::TechnicalSpecification,
        FoldersCategory::ContractDocuments,
    ]
    .iter()
    .for_each(|attachment_type| {
        let folder = request.attachment_list.iter().find(|item| {
            item.kind_id == 2 && item.category_id == Some(*attachment_type)
        });
        let file = folder.and_then(|folder| {
            request.attachment_list.iter().find(|file| {
                file.kind_id == 1
                    && file.parent_id == Some(folder.id)
                    && !file.is_removed
                    && !file.is_classified
            })
        });
        if file.is_none() {
            messages.add_prepared_message(
                CheckRequestPriceInfoMessage::Attachment(*attachment_type)
                    .create_message(),
            );
        }
    });

    if request.item_list.is_empty() {
        messages.add_prepared_message(
            CheckRequestPriceInfoMessage::Fill(String::from("данные спецификации"))
                .create_message(),
        );
    }

    messages
}

#[derive(Debug)]
enum CheckRequestPriceInfoMessage {
    CloseRequestReason,
    Fill(String),
    FillField(String, String),
    Attachment(FoldersCategory),
}

impl CheckRequestPriceInfoMessage {
    fn create_message(self) -> Message {
        match self {
            Self::CloseRequestReason => {
                Message::error("Для закрытого ЗЦИ заполните поле \"Обоснование\"")
                    .with_fields(vec!["request_type_text".to_string()])
            }
            Self::Fill(field_name) => {
                Message::info(format!("Заполните {}", field_name))
            }
            Self::FillField(field_name, field) => {
                Message::info(format!("Заполните поле \"{}\"", field_name))
                    .with_fields(vec![field])
            }
            Self::Attachment(attachment_type) => {
                Message::info(format!("Прикрепите {}", attachment_type))
            }
        }
    }
}
