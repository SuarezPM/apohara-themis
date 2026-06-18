# THEMIS 3.0 - C-12 peer image.
#
# The default CMD runs invoice_validator.py. docker-compose.yml
# overrides it per service for compliance_reviewer.py and
# bench_runner.py. MockPydanticAIAgent.py is stdlib-only and needs
# no install - drop it in via the agents/ mount if a peer image
# without the heavy frameworks is required.
#
# crewai 0.11.2 ships a tiktoken<0.6.0 source wheel that does NOT
# build on Python 3.14. We install crewai `--no-deps` to keep the
# contract honest (the package is "installed") and the bench_runner
# uses its documented mock fallback at runtime. See bench_runner.py.

FROM python:3.14-slim

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1

WORKDIR /app

# System deps: only what pip needs (curl for healthchecks).
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt ./
RUN pip install -r requirements.txt \
    && pip install --no-deps 'crewai>=0.11' || true

COPY . /app

# Default peer. docker-compose.yml overrides per service.
CMD ["python", "invoice_validator.py"]

EXPOSE 7001 7002 7003
