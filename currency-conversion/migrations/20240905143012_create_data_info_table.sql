-- table data info
CREATE TABLE IF NOT EXISTS data_info
(
    data_name text NOT NULL,
    last_update_date timestamp without time zone NOT NULL,
    CONSTRAINT data_info_pkey PRIMARY KEY (data_name)
)
