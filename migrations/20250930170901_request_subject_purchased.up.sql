ALTER TABLE public.request_subject_purchased ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.request_subject_purchased_id_seq
    START WITH 10000
    INCREMENT BY 1
);
