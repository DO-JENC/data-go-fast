INSERT INTO groups (id, name)
VALUES('00000000-0000-0000-0000-000000000001', 'Default Group');

INSERT INTO datasources (id, s3_id, name, file_type, size, created_at, group_id)
VALUES('11111111-1111-1111-1111-111111111111',
'11111111-1111-1111-1111-111111111111',
'Sample Datasource',
'csv',
1024.0,
NOW(), '00000000-0000-0000-0000-000000000001');


INSERT INTO datasources (id, s3_id, name, file_type, size, created_at, group_id)
VALUES('22222222-2222-2222-2222-222222222222',
'22222222-2222-2222-2222-222222222222',
'Sample Datasource',
'json',
1024.0,
NOW(), '00000000-0000-0000-0000-000000000001');
