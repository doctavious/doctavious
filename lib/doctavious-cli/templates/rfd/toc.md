# Requests For Discussion

{% if intro is defined -%}
{{ intro }}

{% endif %}
{%- for entry in entries -%}
* [`{{ entry.description }}`]({{ link_prefix }}/{{ entry.file_path }})
{% endfor -%}
{%- if outro is defined %}
{{ outro }}
{%- endif %}
