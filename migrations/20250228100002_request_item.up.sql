

CREATE TABLE public.request_item (
    uuid uuid NOT NULL,
    request_uuid uuid NOT NULL,
    plan_item_uuid uuid NOT NULL,
    number smallint NOT NULL,
    description_internal character varying(1000) DEFAULT ''::character varying NOT NULL,
    quantity bigint DEFAULT 0 NOT NULL,
    unit_id smallint DEFAULT 0 NOT NULL,
    category_id smallint DEFAULT 0 NOT NULL,
    product_type_id smallint DEFAULT 0 NOT NULL,
    okved2_id integer DEFAULT 0 NOT NULL,
    okpd2_id integer DEFAULT 0 NOT NULL,
    mark character varying(1000),
    technical_requirements character varying(1000),
    delivery_basis character varying(1000) DEFAULT ''::character varying NOT NULL,
    delivery_start_date date NOT NULL,
    delivery_end_date date NOT NULL,
    price bigint DEFAULT 0 NOT NULL,
    sum_excluded_vat bigint DEFAULT 0 NOT NULL,
    vat_id smallint DEFAULT 0 NOT NULL
);

ALTER TABLE ONLY public.request_item
    ADD CONSTRAINT request_item_pkey PRIMARY KEY (uuid);

COMMENT ON TABLE public.request_item IS 'Позиции ЗЦИ';



COMMENT ON COLUMN public.request_item.uuid IS 'UID позиции ЗЦИ';



COMMENT ON COLUMN public.request_item.request_uuid IS 'UID ЗЦИ';



COMMENT ON COLUMN public.request_item.plan_item_uuid IS 'UID позиции ППЗ/ДС';



COMMENT ON COLUMN public.request_item.number IS 'Номер позиции';



COMMENT ON COLUMN public.request_item.description_internal IS 'Наименование позиции';



COMMENT ON COLUMN public.request_item.quantity IS 'Количество';



COMMENT ON COLUMN public.request_item.unit_id IS 'Единица измерения';



COMMENT ON COLUMN public.request_item.category_id IS 'Вид предмета закупки';



COMMENT ON COLUMN public.request_item.product_type_id IS 'Тип позиции';



COMMENT ON COLUMN public.request_item.okved2_id IS 'ОКВЭД2';



COMMENT ON COLUMN public.request_item.okpd2_id IS 'ОКПД2';



COMMENT ON COLUMN public.request_item.mark IS 'Марка, ТУ';



COMMENT ON COLUMN public.request_item.technical_requirements IS 'Технические требования';



COMMENT ON COLUMN public.request_item.delivery_basis IS 'Базис поставки';



COMMENT ON COLUMN public.request_item.delivery_start_date IS 'Дата поставки/начало работ/оказания услуг';



COMMENT ON COLUMN public.request_item.delivery_end_date IS 'Дата окончания выполнения работ/оказания услуг';



COMMENT ON COLUMN public.request_item.price IS 'Цена (без НДС)';



COMMENT ON COLUMN public.request_item.sum_excluded_vat IS 'Стоимость (без НДС)';



COMMENT ON COLUMN public.request_item.vat_id IS 'Ставка НДС';
