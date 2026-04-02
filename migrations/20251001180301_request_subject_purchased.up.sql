ALTER TABLE public.request_subject_purchased ALTER COLUMN id DROP IDENTITY;

CREATE SEQUENCE public.request_subject_purchased_id_seq INCREMENT BY 1 MINVALUE 1;

SELECT setval('public.request_subject_purchased_id_seq', COALESCE((SELECT MAX(id) + 1 FROM public.request_subject_purchased), 1));

ALTER TABLE public.request_subject_purchased ALTER COLUMN id SET DEFAULT nextval('public.request_subject_purchased_id_seq');
