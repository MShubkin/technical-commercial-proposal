

CREATE TABLE public.status_history (
    uuid uuid NOT NULL,
    object_uuid uuid NOT NULL,
    tcp_status_type smallint,
    status_id smallint NOT NULL,
    created_at timestamp without time zone NOT NULL,
    created_by integer NOT NULL
);

ALTER TABLE ONLY public.status_history
    ADD CONSTRAINT status_history_pkey PRIMARY KEY (uuid);


