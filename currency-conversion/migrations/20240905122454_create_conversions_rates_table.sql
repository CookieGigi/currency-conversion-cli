-- table conversions rates
CREATE TABLE conversions_rates
(
    id uuid NOT NULL,
    "from" text NOT NULL,
    "to" text NOT NULL,
    rate numeric NOT NULL,
    PRIMARY KEY (id)
);
