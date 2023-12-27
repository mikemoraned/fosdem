-- this produces some output which looks sensible (find Ceph-related content):
SELECT * FROM embedding_1 WHERE id != 20 ORDER BY embedding <-> (SELECT embedding FROM embedding_1 WHERE id = 20) LIMIT 5;
