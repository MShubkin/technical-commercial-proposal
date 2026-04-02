ALTER TABLE public.request_subject_purchased ALTER COLUMN id DROP IDENTITY;

DROP SEQUENCE public.request_subject_purchased_id_seq;
