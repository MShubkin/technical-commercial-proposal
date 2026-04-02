ALTER TABLE proposal_head
DROP COLUMN IF EXISTS etp_uuid;

ALTER TABLE proposal_head 
ADD COLUMN etp_id integer NULL;
