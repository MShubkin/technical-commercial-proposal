

CREATE TABLE public.request_partner (
    uuid uuid NOT NULL,
    request_uuid uuid NOT NULL,
    supplier_id integer DEFAULT 0 NOT NULL,
    number smallint DEFAULT 0 NOT NULL,
    is_public boolean DEFAULT false NOT NULL,
    is_phone_check boolean DEFAULT false NOT NULL,
    is_email_check boolean DEFAULT false NOT NULL,
    resume character varying(2000),
    comment character varying(2000),
    is_removed boolean DEFAULT false NOT NULL
);


ALTER TABLE ONLY public.request_partner
    ADD CONSTRAINT "REQUEST_PARTNER_pkey" PRIMARY KEY (uuid);

COMMENT ON TABLE public.request_partner IS 'Атрибуты поставщиков запроса ценовой информации';



COMMENT ON COLUMN public.request_partner.uuid IS 'UID записи';



COMMENT ON COLUMN public.request_partner.request_uuid IS 'UID ЗЦИ';



COMMENT ON COLUMN public.request_partner.supplier_id IS 'Поставщик';



COMMENT ON COLUMN public.request_partner.number IS 'Порядковый номер записи в рамках ЗЦИ';



COMMENT ON COLUMN public.request_partner.is_public IS 'Публикуется на ЭТП ГПБ';



COMMENT ON COLUMN public.request_partner.is_phone_check IS 'Телефонные переговоры';



COMMENT ON COLUMN public.request_partner.is_email_check IS 'Отправка материалов по электронной почте';



COMMENT ON COLUMN public.request_partner.resume IS 'Результат коммуникаций';



COMMENT ON COLUMN public.request_partner.comment IS 'Комментарий, текущая ситуация';



COMMENT ON COLUMN public.request_partner.is_removed IS 'Метка удаления';
