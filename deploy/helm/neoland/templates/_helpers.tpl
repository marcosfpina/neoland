{{/*
Expand the name of the chart.
*/}}
{{- define "neoland.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "neoland.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "neoland.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "neoland.labels" -}}
helm.sh/chart: {{ include "neoland.chart" . }}
{{ include "neoland.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "neoland.selectorLabels" -}}
app.kubernetes.io/name: {{ include "neoland.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "neoland.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "neoland.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Database URL from postgresql subchart or external
*/}}
{{- define "neoland.databaseUrl" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "postgresql://%s:%s@%s-postgresql:5432/%s" .Values.postgresql.auth.username .Values.postgresql.auth.password (include "neoland.fullname" .) .Values.postgresql.auth.database }}
{{- else }}
{{- required "database.url must be set when postgresql.enabled is false" .Values.config.databaseUrl }}
{{- end }}
{{- end }}

{{/*
JWT secret key (32 bytes for HS256)
*/}}
{{- define "neoland.jwtSecret" -}}
{{- .Values.config.auth.jwtSecret | default "neoland-dev-jwt-secret-change-in-production" }}
{{- end }}
