# TIL
> Today I Learned

* TILs: {{ til_count }}
* Topics: {{ categories_count }}

{% for key, value in tils.items() %}
  ## {{ key }}
  {% for til in value %}
    * [`{{ til.title }}`]{{ til.topic}}/{{ til.file_name }}) {{ til.description }} ({{ til.date }})
  {% endfor %}

{% endfor %}