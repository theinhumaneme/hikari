CREATE TABLE deploy_config (
                               id BIGSERIAL PRIMARY KEY,
                               client TEXT NOT NULL,
                               environment TEXT NOT NULL,
                               solution TEXT NOT NULL
);
CREATE TABLE compose_stack (
                              id BIGSERIAL PRIMARY KEY,
                              deployment_id BIGINT NOT NULL REFERENCES deploy_config(id) ON DELETE CASCADE,
                              stack_name TEXT NOT NULL,
                              filename TEXT NOT NULL,
                              home_directory TEXT NOT NULL
);
CREATE TABLE container (
                           id BIGSERIAL PRIMARY KEY,
                           stack_id BIGINT NOT NULL REFERENCES compose_stack(id) ON DELETE CASCADE,
                           service_name TEXT NOT NULL,
                           container_name TEXT NOT NULL,
                           image TEXT NOT NULL,
                           restart TEXT NOT NULL,
                           "user" TEXT,
                           stdin_open BOOLEAN,
                           tty BOOLEAN,
                           command TEXT,
                           pull_policy TEXT,
                           ports TEXT[],
                           volumes TEXT[],
                           environment TEXT[],
                           mem_reservation TEXT,
                           mem_limit TEXT,
                           oom_kill_disable BOOLEAN,
                           privileged BOOLEAN
);
CREATE INDEX idx_compose_stack_deployment_id ON compose_stack(deployment_id);
CREATE INDEX idx_container_stack_id ON container(stack_id);
