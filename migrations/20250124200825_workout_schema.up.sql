-- Add up migration script here
DROP TABLE IF EXISTS `exercises`;
CREATE TABLE `exercises` (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    exercise_category VARCHAR(16),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

DROP TABLE IF EXISTS `routines`;
CREATE TABLE `routines` (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    user_id BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

DROP TABLE IF EXISTS `routine_exercises`;
CREATE TABLE `routine_exercises` (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    routine_id BIGINT NOT NULL,
    exercise_id BIGINT NOT NULL,
    sets INT NOT NULL DEFAULT 3,
    reps INT NOT NULL DEFAULT 10,
    weight_kg DECIMAL(5,2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (routine_id) REFERENCES routines(id),
    FOREIGN KEY (exercise_id) REFERENCES exercises(id)
);

DROP TABLE IF EXISTS `workout_logs`;
CREATE TABLE `workout_logs` (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    routine_id BIGINT NOT NULL,
    start_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    end_time TIMESTAMP,
    notes TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (routine_id) REFERENCES routines(id)
);

DROP TABLE IF EXISTS `exercise_sets`;
CREATE TABLE `exercise_sets` (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    workout_log_id BIGINT NOT NULL,
    exercise_id BIGINT NOT NULL,
    set_number INT NOT NULL,
    reps_completed INT NOT NULL,
    weight_kg DECIMAL(5,2),
    notes TEXT,
    completed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (workout_log_id) REFERENCES workout_logs(id),
    FOREIGN KEY (exercise_id) REFERENCES exercises(id)
);

INSERT INTO exercises (name, exercise_category) 
VALUES 
	('Bench Press', 'Chest'),
	('Include Bench Press', 'Chest'),
	('Dumbell Bench Press', 'Chest'),
	('Dumbell Fly', 'Chest'),
	('Cable Fly', 'Chest'),
	('Leg Press', 'Leg'),
	('Barbell Back Squat', 'Legs'),
	('Barbell Front Squat', 'Legs'),
	('Dumbell Curl', 'Arms'),
	('Barbell Curl', 'Arms'),
	('Tricep Pushdown', 'Arms'),
	('Tricep Extension', 'Arms'),
	('Barbell Row', 'Back'),
	('Lat Pulldown', 'Back');
