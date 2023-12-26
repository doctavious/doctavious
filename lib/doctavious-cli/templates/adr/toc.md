# Architecture Decision Records

{% if intro is defined -%}
{{ intro }}

{% endif %}
{%- for adr in adrs -%}
* [`{{ adr.description }}`]({{ link_prefix }}/{{ adr.file_path }})
{% endfor -%}
{%- if outro is defined %}
{{ outro }}
{%- endif %}
