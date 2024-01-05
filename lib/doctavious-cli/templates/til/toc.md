# TIL
> Today I Learned

* TILs: {{ til_count }}
* Topics: {{ categories_count }}

{% for key in tils -%}
## {{ key }}
  
{% for til in tils[key] -%}
* [`{{ til.title }}`]({{ til.topic}}/{{ til.file_name }}) {{ til.description }} ({{ til.date }})
{% endfor %}

{% endfor %}