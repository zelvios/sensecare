CREATE TABLE roles
(
    id          smallint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name        varchar(32)  NOT NULL UNIQUE,
    description varchar(255) NOT NULL
);

INSERT INTO roles (name, description)
VALUES ('client', 'Patient eller bruger med adgang til eget værelse'),
       ('staff', 'Personale med adgang til samtlige værelser, tilkald og alarmer'),
       ('admin', 'Administrator, der kan administrere værelser, enheder og brugerkonti');
