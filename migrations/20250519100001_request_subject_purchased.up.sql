

CREATE TABLE public.request_subject_purchased
(
    uuid uuid NOT NULL PRIMARY KEY,
    id BIGINT NOT NULL,
    organization_unit_id SMALLINT NOT NULL DEFAULT 0,
    hierarchy_id SMALLINT NOT NULL,
    hierarchy_uuid uuid NOT NULL,
    contract_subject_purchase_text VARCHAR(2000) NOT NULL,
    parent_uuid uuid,
    is_removed BOOLEAN NOT NULL DEFAULT false,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_by INTEGER NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL
);

COMMENT ON TABLE public.request_subject_purchased
    IS 'Предметы закупки ЗЦИ';

COMMENT ON COLUMN public.request_subject_purchased.uuid
    IS 'Уникальный идентификатор записи';
COMMENT ON COLUMN public.request_subject_purchased.id
    IS 'Идентификатор Предмета закупки/Группы предметов закупки';
COMMENT ON COLUMN public.request_subject_purchased.organization_unit_id
    IS 'Подразделение';
COMMENT ON COLUMN public.request_subject_purchased.hierarchy_id
    IS 'Уровень иерархии';
COMMENT ON COLUMN public.request_subject_purchased.hierarchy_uuid
    IS 'Уникальный идентификатор вышестоящей записи';
COMMENT ON COLUMN public.request_subject_purchased.contract_subject_purchase_text
    IS 'Наименование предмета закупки';
COMMENT ON COLUMN public.request_subject_purchased.parent_uuid
    IS 'Уникальный идентификатор родительской записи';
COMMENT ON COLUMN public.request_subject_purchased.is_removed
    IS 'Признак удаления';
COMMENT ON COLUMN public.request_subject_purchased.created_by
    IS 'Создал';
COMMENT ON COLUMN public.request_subject_purchased.created_at
    IS 'Дата создания';
COMMENT ON COLUMN public.request_subject_purchased.changed_by
    IS 'Изменил';
COMMENT ON COLUMN public.request_subject_purchased.changed_at
    IS 'Дата изменения';
