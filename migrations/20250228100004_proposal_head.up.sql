

CREATE TABLE public.proposal_head (
    uuid uuid NOT NULL,
    id bigint NOT NULL,
    etp_uuid uuid,
    request_uuid uuid NOT NULL,
    supplier_uuid uuid NOT NULL,
    start_date date,
    end_date date,
    currency_id integer DEFAULT 0 NOT NULL,
    status_id smallint DEFAULT 0 NOT NULL,
    status_check_id smallint DEFAULT 0 NOT NULL,
    result_id smallint,
    receive_date timestamp without time zone,
    proposal_source smallint,
    sum_excluded_vat_total bigint,
    contact_phone character varying(100),
    created_by integer NOT NULL,
    changed_by integer NOT NULL,
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL,
    hierarchy_uuid uuid
);

ALTER TABLE public.proposal_head ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.proposal_head_id_seq
    START WITH 6500000000
    INCREMENT BY 1
    MINVALUE 6500000000
    MAXVALUE 6599999999
    CACHE 1
);

ALTER TABLE ONLY public.proposal_head
    ADD CONSTRAINT proposal_head_pkey PRIMARY KEY (uuid);


COMMENT ON TABLE public.proposal_head IS 'Атрибуты заголовка ТКП';



COMMENT ON COLUMN public.proposal_head.uuid IS 'UID ТКП ';



COMMENT ON COLUMN public.proposal_head.id IS 'Номер ТКП ';



COMMENT ON COLUMN public.proposal_head.etp_uuid IS 'Уникальный идентификатор ТКП ЭТП ГПБ';



COMMENT ON COLUMN public.proposal_head.request_uuid IS 'UID ЗЦИ';



COMMENT ON COLUMN public.proposal_head.supplier_uuid IS 'UID организации';



COMMENT ON COLUMN public.proposal_head.start_date IS 'Начало срока действия ';



COMMENT ON COLUMN public.proposal_head.end_date IS 'Окончание срока действия ';



COMMENT ON COLUMN public.proposal_head.currency_id IS 'Валюта';



COMMENT ON COLUMN public.proposal_head.status_id IS 'Статус общий';



COMMENT ON COLUMN public.proposal_head.status_check_id IS 'Статус рассмотрения';



COMMENT ON COLUMN public.proposal_head.result_id IS 'Результат рассмотрения';



COMMENT ON COLUMN public.proposal_head.receive_date IS 'Дата поступления ТКП';



COMMENT ON COLUMN public.proposal_head.proposal_source IS 'Источник инфо об организации';



COMMENT ON COLUMN public.proposal_head.sum_excluded_vat_total IS 'Стоимость организации';



COMMENT ON COLUMN public.proposal_head.contact_phone IS 'Контактный телефон ';



COMMENT ON COLUMN public.proposal_head.created_by IS 'Создал';



COMMENT ON COLUMN public.proposal_head.changed_by IS 'Изменил';



COMMENT ON COLUMN public.proposal_head.created_at IS 'Дата создания';



COMMENT ON COLUMN public.proposal_head.changed_at IS 'Дата изменения';
