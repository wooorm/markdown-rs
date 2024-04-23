#![allow(clippy::needless_raw_string_hashes)]

// To do: clippy introduced this in 1.72 but breaks when it fixes it.
// Remove when solved.

use markdown::{message, to_html_with_options, CompileOptions, Options};
use pretty_assertions::assert_eq;

#[test]
fn gfm_tagfilter() -> Result<(), message::Message> {
    assert_eq!(
        to_html_with_options(
            "<iframe>",
            &Options {
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<iframe>",
        "should not filter by default"
    );

    assert_eq!(
        to_html_with_options(
            "a <i>\n<script>",
            &Options {
                compile: CompileOptions {
                    gfm_tagfilter: true,
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<p>a &lt;i&gt;</p>\n&lt;script&gt;",
        "should not turn `allow_dangerous_html` on"
    );

    assert_eq!(
        to_html_with_options(
            "<iframe>",
            &Options {
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    gfm_tagfilter: true,
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "&lt;iframe>",
        "should filter"
    );

    assert_eq!(
        to_html_with_options(
            "<iframe\n>",
            &Options {
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    gfm_tagfilter: true,
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "&lt;iframe\n>",
        "should filter when followed by a line ending (1)"
    );

    assert_eq!(
        to_html_with_options(
            "<div\n>",
            &Options {
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    gfm_tagfilter: true,
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<div\n>",
        "should filter when followed by a line ending (2)"
    );

    assert_eq!(
        to_html_with_options(
            r##"
<title>

<div title="<title>"></div>

<span title="<title>"></span>

<div><title></title></div>

<span><title></title></span>

<b><textarea></textarea></b>

<script/src="#">

<SCRIPT SRC=http://xss.rocks/xss.js></SCRIPT>

<IMG SRC="javascript:alert('XSS');">

<IMG SRC=javascript:alert('XSS')>

<IMG SRC=`javascript:alert("RSnake says, 'XSS'")`>

<IMG """><SCRIPT>alert("XSS")</SCRIPT>"\>

<SCRIPT/XSS SRC="http://xss.rocks/xss.js"></SCRIPT>

<BODY onload!#$%&()*~+-_.,:;?@[/|\]^`=alert("XSS")>

<<SCRIPT>alert("XSS");//\<</SCRIPT>

<SCRIPT SRC=http://xss.rocks/xss.js?< B >

<SCRIPT SRC=//xss.rocks/.j>

</TITLE><SCRIPT>alert("XSS");</SCRIPT>

<STYLE>li {list-style-image: url("javascript:alert('XSS')");}</STYLE><UL><LI>XSS</br>

javascript:/*--></title></style></textarea></script></xmp><svg/onload='+/"/+/onmouseover=1/+/[*/[]/+alert(1)//'>

<STYLE>@import'http://xss.rocks/xss.css';</STYLE>
"##,
            &Options {
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    gfm_tagfilter: true,
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        r###"&lt;title>
<div title="&lt;title>"></div>
<p><span title="&lt;title>"></span></p>
<div>&lt;title>&lt;/title></div>
<p><span>&lt;title>&lt;/title></span></p>
<p><b>&lt;textarea>&lt;/textarea></b></p>
<p>&lt;script/src=&quot;#&quot;&gt;</p>
&lt;SCRIPT SRC=http://xss.rocks/xss.js>&lt;/SCRIPT>
<IMG SRC="javascript:alert('XSS');">
<p>&lt;IMG SRC=javascript:alert('XSS')&gt;</p>
<p>&lt;IMG SRC=<code>javascript:alert(&quot;RSnake says, 'XSS'&quot;)</code>&gt;</p>
<p>&lt;IMG &quot;&quot;&quot;&gt;&lt;SCRIPT>alert(&quot;XSS&quot;)&lt;/SCRIPT>&quot;&gt;</p>
<p>&lt;SCRIPT/XSS SRC=&quot;http://xss.rocks/xss.js&quot;&gt;&lt;/SCRIPT></p>
<BODY onload!#$%&()*~+-_.,:;?@[/|\]^`=alert("XSS")>
<p>&lt;&lt;SCRIPT>alert(&quot;XSS&quot;);//&lt;&lt;/SCRIPT></p>
&lt;SCRIPT SRC=http://xss.rocks/xss.js?< B >

&lt;SCRIPT SRC=//xss.rocks/.j>

&lt;/TITLE>&lt;SCRIPT>alert("XSS");&lt;/SCRIPT>
&lt;STYLE>li {list-style-image: url("javascript:alert('XSS')");}&lt;/STYLE><UL><LI>XSS</br>
<p>javascript:/<em>--&gt;&lt;/title>&lt;/style>&lt;/textarea>&lt;/script>&lt;/xmp>&lt;svg/onload='+/&quot;/+/onmouseover=1/+/[</em>/[]/+alert(1)//'&gt;</p>
&lt;STYLE>@import'http://xss.rocks/xss.css';&lt;/STYLE>
"###,
        "should handle things like GitHub"
    );

    Ok(())
}
