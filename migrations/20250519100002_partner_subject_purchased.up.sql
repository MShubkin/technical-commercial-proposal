

CREATE TABLE public.partner_subject_purchased
(
    uuid uuid NOT NULL PRIMARY KEY,
    uuid_subject uuid NOT NULL,
    supplier_id INTEGER NOT NULL,
    is_removed BOOLEAN NOT NULL DEFAULT false,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_by INTEGER NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL
);

COMMENT ON TABLE public.partner_subject_purchased
    IS 'Организации предметов закупки';

COMMENT ON COLUMN public.partner_subject_purchased.uuid
    IS 'Уникальный идентификатор записи';
COMMENT ON COLUMN public.partner_subject_purchased.uuid_subject
    IS 'Уникальный идентификатор предмета закупки';
COMMENT ON COLUMN public.partner_subject_purchased.supplier_id
    IS 'Организация';
COMMENT ON COLUMN public.partner_subject_purchased.is_removed
    IS 'Признак удаления';
COMMENT ON COLUMN public.partner_subject_purchased.created_by
    IS 'Создал';
COMMENT ON COLUMN public.partner_subject_purchased.created_at
    IS 'Дата создания';
COMMENT ON COLUMN public.partner_subject_purchased.changed_by
    IS 'Изменил';
COMMENT ON COLUMN public.partner_subject_purchased.changed_at
    IS 'Дата изменения';
