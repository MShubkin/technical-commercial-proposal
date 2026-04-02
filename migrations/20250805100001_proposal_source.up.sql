ALTER TABLE public.proposal_head 
ALTER COLUMN proposal_source TYPE VARCHAR(10) 
USING proposal_source::VARCHAR(10);
