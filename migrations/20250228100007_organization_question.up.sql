

CREATE TABLE public.organization_question (
    uuid uuid NOT NULL,
    question_text character varying NOT NULL,
    answer_uuid uuid,
    answer_question_text character varying,
    request_uuid uuid NOT NULL,
    supplier_uuid uuid NOT NULL,
    created_by integer NOT NULL,
    question_created_at timestamp without time zone NOT NULL,
    answer_created_at timestamp without time zone,
    supplier_id integer NOT NULL,
    answer_published_at timestamp without time zone
);

ALTER TABLE ONLY public.organization_question
    ADD CONSTRAINT organization_question_pkey PRIMARY KEY (uuid);
