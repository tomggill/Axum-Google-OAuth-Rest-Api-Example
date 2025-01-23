INSERT INTO sessions (session_id, csrf_token, expires_at) VALUES 
    ("test_session_id", "test_csrf_token", NOW() + INTERVAL 1 HOUR),
    ("expired_session_id", "test_csrf_token", DATE_SUB(NOW(), INTERVAL 10 MINUTE));
    