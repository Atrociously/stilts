use crate::Root;

const TEMPLATE: &str = r###"
{% extends "base.html" %}

{% fn my_func(s: &str) -> String {
    let mut out = "OOF".to_string();
    out.push_str(s);
    out
} %}

{% macro my_mac(time: std::time::Duration) %}
    INSIDE MY MAC
{% end %}

{% block head %}
    {% a %}
    {% super() %}
    overwrites
{% end %}

{% block header %}
    {% for i in 0..10 %}
        {% match i %}
            {% when 2 if i != 0 %}
                {% i.json() %}
            {% when 3 | 4 %}
                {% a %}
            {% when _ %}
        {% end %}
    {% end %}
    {% if true %}
        {% a %}
    {% else %}
        {% a %}
    {% end %}
{% end %}

{% block main %}
    {% "Hello Word" %}
    {% include "other.html" %}
    {% a %}
{% end %}

{% block footer %}
    {% call my_mac(std::time::Duration::from_secs(50)) %}
    {% let s = r#"MYSTR%}"#; %}
    {% my_func(s) %}
{% end %}
"###;

#[test]
pub fn parse_example_template() {
    Root::parse(TEMPLATE).unwrap();
}
