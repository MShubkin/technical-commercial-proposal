SELECT setval('public.request_subject_purchased_id_seq', (SELECT MAX(id) FROM public.request_subject_purchased));
