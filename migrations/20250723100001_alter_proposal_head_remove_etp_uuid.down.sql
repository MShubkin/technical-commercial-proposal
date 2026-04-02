ALTER TABLE proposal_head 
DROP COLUMN IF EXISTS etp_id;

ALTER TABLE proposal_head
ADD COLUMN etp_uuid uuid NULL;
