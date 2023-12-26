# Architecture Decision Records

{% if intro is defined -%}
{{ intro }}

{% endif %}
{%- for adr in adrs -%}
* [`{{ adr.description }}`]({{ adr.link_prefix}}/{{ adr.path }})
{% endfor -%}
{%- if outro is defined %}
{{ outro }}
{%- endif %}
