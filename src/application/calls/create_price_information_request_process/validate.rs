use shared_essential::presentation::dto::response_request::MessageKind;
use shared_essential::presentation::dto::technical_commercial_proposal::create_price_information_request::CreatePriceInformationRequest;
use crate::common::{Validate, ValidationResults};
use shared_essential::domain::tables::tcp::PriceInformationRequestType;

impl Validate for CreatePriceInformationRequest {
    /// Валидация пользовательских данных при создании ЗЦИ
    fn validate(&self) -> ValidationResults {
        let mut results = ValidationResults::default();

        check_required(
            &mut results,
            &self.period_of_validity.to_string(),
            "Срок действия",
        );
        check_required(
            &mut results,
            &self.technical_specification.uuid,
            "Техническое задание",
        );
        check_required(&mut results, &self.draft_treaty.uuid, "Проект договора");
        check_required(&mut results, &self.template_tkp.uuid, "Шаблон ТКП");

        if self.request_type == PriceInformationRequestType::Private as i16
            && (self.suppliers.is_none()
                || self.suppliers.as_ref().unwrap().is_empty())
        {
            check_required(&mut results, "", "Поставщики");
        }
        results
    }
}

fn check_required(results: &mut ValidationResults, value: &str, field_name: &str) {
    if value.is_empty() {
        results.messages.add_message(
            MessageKind::Error,
            format!("Заполните поле \"{}\"", field_name),
        );
    }
}
