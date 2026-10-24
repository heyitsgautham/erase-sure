# Verification portal (FastAPI). Build from the repository root:
#   docker build -f portal/Dockerfile -t securewipe-portal .
FROM python:3.12-slim

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1

WORKDIR /srv

COPY portal/requirements-lock.txt portal/requirements-lock.txt
RUN pip install --require-hashes -r portal/requirements-lock.txt

# app/main.py loads schemas from ../../certs/schemas relative to itself.
COPY certs/schemas certs/schemas
COPY portal/app portal/app
COPY portal/run.py portal/keygen.py portal/

RUN useradd --system --uid 10001 --no-create-home securewipe
USER securewipe
WORKDIR /srv/portal

EXPOSE 8000
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s \
    CMD python -c "import urllib.request; urllib.request.urlopen('http://127.0.0.1:8000/health', timeout=2)"

CMD ["python", "run.py", "--host", "0.0.0.0", "--port", "8000"]
