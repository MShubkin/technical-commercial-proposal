

CREATE TABLE public.request_head (
    uuid uuid NOT NULL,
    id bigint NOT NULL,
    plan_uuid uuid,
    plan_id bigint,
    hierarchy_uuid uuid,
    type_request_id smallint,
    request_subject character varying(2000),
    start_date timestamp without time zone,
    end_date timestamp without time zone,
    status_id smallint NOT NULL,
    customer_id integer,
    currency_id smallint,
    request_type_text character varying(2000),
    organizer_id integer,
    organizer_name character varying(200),
    organizer_mail character varying(200),
    organizer_phone character varying(200),
    organizer_location character varying(200),
    reason_closing character varying(200),
    purchasing_trend_id smallint,
    created_by integer NOT NULL,
    changed_by integer NOT NULL,
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL
);

ALTER TABLE public.request_head ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.request_head_id_seq
    START WITH 2000000000
    INCREMENT BY 1
    MINVALUE 2000000000
    MAXVALUE 2999999999
    CACHE 1
);

ALTER TABLE ONLY public.request_head
    ADD CONSTRAINT request_head_pkey PRIMARY KEY (uuid);

COMMENT ON TABLE public.request_head IS 'Атрибуты заголовка ЗЦИ';



COMMENT ON COLUMN public.request_head.uuid IS 'UID ЗЦИ';



COMMENT ON COLUMN public.request_head.id IS 'Номер ЗЦИ';



COMMENT ON COLUMN public.request_head.plan_uuid IS 'UID ППЗ/ДС';



COMMENT ON COLUMN public.request_head.plan_id IS 'Номер ППЗ/ДС';



COMMENT ON COLUMN public.request_head.hierarchy_uuid IS 'UUID иерархии документов';



COMMENT ON COLUMN public.request_head.type_request_id IS 'Тип ЗЦИ';



COMMENT ON COLUMN public.request_head.request_subject IS 'Предмет ЗЦИ';



COMMENT ON COLUMN public.request_head.start_date IS 'Дата и время начала сбора ТКП';



COMMENT ON COLUMN public.request_head.end_date IS 'Дата и время окончания сбора ТКП';



COMMENT ON COLUMN public.request_head.status_id IS 'Текущий статус ЗЦИ';



COMMENT ON COLUMN public.request_head.customer_id IS 'Заказчик';



COMMENT ON COLUMN public.request_head.currency_id IS 'Валюта';



COMMENT ON COLUMN public.request_head.request_type_text IS 'Обоснование закрытого ЗЦИ';



COMMENT ON COLUMN public.request_head.organizer_id IS 'Организатор ЗЦИ';



COMMENT ON COLUMN public.request_head.organizer_name IS 'Контактное лицо';



COMMENT ON COLUMN public.request_head.organizer_mail IS 'Электронный адрес';



COMMENT ON COLUMN public.request_head.organizer_phone IS 'Телефон';



COMMENT ON COLUMN public.request_head.organizer_location IS 'Местонахождение';



COMMENT ON COLUMN public.request_head.reason_closing IS 'Причина досрочного закрытия';



COMMENT ON COLUMN public.request_head.purchasing_trend_id IS 'Направление закупки';



COMMENT ON COLUMN public.request_head.created_by IS 'Создал';



COMMENT ON COLUMN public.request_head.changed_by IS 'Изменил';



COMMENT ON COLUMN public.request_head.created_at IS 'Дата создания';



COMMENT ON COLUMN public.request_head.changed_at IS 'Дата изменения';
