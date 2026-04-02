ALTER TABLE public.proposal_head 
ALTER COLUMN proposal_source TYPE SMALLINT
USING proposal_source::SMALLINT;
