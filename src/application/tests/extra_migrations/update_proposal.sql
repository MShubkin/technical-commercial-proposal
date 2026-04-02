INSERT INTO proposal_head(
    uuid,
    request_uuid,
    supplier_uuid,
    start_date,
    end_date,
    created_by,
    changed_by,
    created_at,
    changed_at,
    receive_date
) values
        ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001', now()::date, now()::date, 1, 1, now()::timestamp, now()::timestamp, now()::timestamp);

INSERT INTO proposal_item(
    uuid,
    "number",
    proposal_uuid,
    request_item_uuid
) values
        ('00000000-0000-0000-0000-000000000001', 1, '00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001');

INSERT INTO request_partner(uuid, request_uuid, supplier_id, "number", is_public, is_phone_check, is_email_check, is_removed) values
        -- Публичный партнер
        ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 1, 1, true, false, false, false),
        ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000002', 2, 1, false, false, false, false);

INSERT INTO public.request_item(
    uuid,
    number,
    request_uuid,
    plan_item_uuid,
    delivery_start_date,
    delivery_end_date,
    price,
    vat_id
) VALUES
('00000000-0000-0000-0000-000000000001', 1, '00000000-0000-0000-0000-000000000001','81000000-0000-0000-0000-000000000001', '1999-01-01','1999-10-10', 500, 11),
('00000000-0000-0000-0000-000000000002', 2, '00000000-0000-0000-0000-000000000001','81000000-0000-0000-0000-000000000002', '1999-01-01','1999-10-10', 5, 12);
