

CREATE TABLE public.proposal_item (
    uuid uuid NOT NULL,
    number smallint NOT NULL,
    proposal_uuid uuid NOT NULL,
    request_item_uuid uuid NOT NULL,
    description_internal character varying(1000) DEFAULT ''::character varying NOT NULL,
    quantity bigint DEFAULT 0 NOT NULL,
    unit_id integer DEFAULT 0 NOT NULL,
    price bigint,
    vat_id smallint,
    sum_included_vat bigint,
    sum_excluded_vat bigint,
    manufacturer character varying(200),
    mark character varying(200),
    pay_condition_id smallint,
    prepayment_percent smallint,
    delivery_condition character varying(200),
    execution_percent smallint,
    is_possibility boolean DEFAULT true NOT NULL,
    possibility_note character varying(200),
    analog_description character varying(200),
    delivery_period character varying(50)
);

ALTER TABLE ONLY public.proposal_item
    ADD CONSTRAINT proposal_item_pkey PRIMARY KEY (uuid);

COMMENT ON TABLE public.proposal_item IS 'Атрибуты позиций ТКП';



COMMENT ON COLUMN public.proposal_item.uuid IS 'UID позиции ТКП';



COMMENT ON COLUMN public.proposal_item.number IS 'Номер позиции';



COMMENT ON COLUMN public.proposal_item.proposal_uuid IS 'UID ТКП';



COMMENT ON COLUMN public.proposal_item.request_item_uuid IS 'UID позиции ЗЦИ';



COMMENT ON COLUMN public.proposal_item.description_internal IS 'Наименование позиции';



COMMENT ON COLUMN public.proposal_item.quantity IS 'Количество от Организации';



COMMENT ON COLUMN public.proposal_item.unit_id IS 'Единица измерения Организации';



COMMENT ON COLUMN public.proposal_item.price IS 'Цена Организации (без НДС)';



COMMENT ON COLUMN public.proposal_item.vat_id IS 'Ставка НДС Организации';



COMMENT ON COLUMN public.proposal_item.sum_included_vat IS 'Стоимость Организации (c НДС)';



COMMENT ON COLUMN public.proposal_item.sum_excluded_vat IS 'Стоимость Организации (без НДС)';



COMMENT ON COLUMN public.proposal_item.manufacturer IS 'Наименование производителя';



COMMENT ON COLUMN public.proposal_item.mark IS 'Тип, марка продукции';



COMMENT ON COLUMN public.proposal_item.pay_condition_id IS 'Условия оплаты';



COMMENT ON COLUMN public.proposal_item.prepayment_percent IS 'Размер аванса, %';



COMMENT ON COLUMN public.proposal_item.delivery_condition IS 'Условия поставки';



COMMENT ON COLUMN public.proposal_item.execution_percent IS '% выполнения собственными силами';



COMMENT ON COLUMN public.proposal_item.is_possibility IS 'Возможность поставки';



COMMENT ON COLUMN public.proposal_item.possibility_note IS 'Причина невозможности поставки';



COMMENT ON COLUMN public.proposal_item.analog_description IS 'Техническое описание предлагаемого эквивалента';



COMMENT ON COLUMN public.proposal_item.delivery_period IS 'Срок поставки';
