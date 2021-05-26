/// Default css styles provided by vimwiki
pub static DEFAULT_STYLES_FILE: &str = r#"
body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Roboto", "Oxygen", "Ubuntu", "Cantarell", "Fira Sans", "Droid Sans", "Helvetica Neue", sans-serif;;
  margin: 2em 4em 2em 4em;
  font-size: 120%;
  line-height: 130%;
}

h1, h2, h3, h4, h5, h6 {
  font-weight: bold;
  line-height:100%;
  margin-top: 1.5em;
  margin-bottom: 0.5em;
}

h1 {font-size: 2em; color: #000000;}
h2 {font-size: 1.8em; color: #404040;}
h3 {font-size: 1.6em; color: #707070;}
h4 {font-size: 1.4em; color: #909090;}
h5 {font-size: 1.2em; color: #989898;}
h6 {font-size: 1em; color: #9c9c9c;}

p, pre, blockquote, table, ul, ol, dl {
  margin-top: 1em;
  margin-bottom: 1em;
}

ul ul, ul ol, ol ol, ol ul {
  margin-top: 0.5em;
  margin-bottom: 0.5em;
}

li { margin: 0.3em auto; }

ul {
  margin-left: 2em;
  padding-left: 0;
}

dt { font-weight: bold; }

img { border: none; }

pre {
  border-left: 5px solid #dcdcdc;
  background-color: #f5f5f5;
  padding-left: 1em;
  font-family: Monaco, "Courier New", "DejaVu Sans Mono", "Bitstream Vera Sans Mono", monospace;
  font-size: 0.8em;
  border-radius: 6px;
}

p > a {
  color: white;
  text-decoration: none;
  font-size: 0.7em;
  padding: 3px 6px;
  border-radius: 3px;
  background-color: #1e90ff;
  text-transform: uppercase;
  font-weight: bold;
}

p > a:hover {
  color: #dcdcdc;
  background-color: #484848;
}

li > a {
  color: #1e90ff;
  font-weight: bold;
  text-decoration: none;
}

li > a:hover { color: #ff4500; }

blockquote {
  color: #686868;
  font-size: 0.8em;
  line-height: 120%;
  padding: 0.8em;
  border-left: 5px solid #dcdcdc;
}

th, td {
  border: 1px solid #ccc;
  padding: 0.3em;
}

th { background-color: #f0f0f0; }

hr {
  border: none;
  border-top: 1px solid #ccc;
  width: 100%;
}

del {
  text-decoration: line-through;
  color: #777777;
}

.toc li { list-style-type: none; }

.todo {
  font-weight: bold;
  background-color: #ff4500 ;
  color: white;
  font-size: 0.8em;
  padding: 3px 6px;
  border-radius: 3px;
}

.justleft { text-align: left; }
.justright { text-align: right; }
.justcenter { text-align: center; }

.center {
  margin-left: auto;
  margin-right: auto;
}

.tag {
  background-color: #eeeeee;
  font-family: monospace;
  padding: 2px;
}

.header a {
  text-decoration: none;
  color: inherit;
}

/* classes for items of todo lists */

.rejected {
  /* list-style: none; */
  background-image: url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA8AAAAPCAMAAAAMCGV4AAAACXBIWXMAAADFAAAAxQEdzbqoAAAAB3RJTUUH4QgEFhAtuWgv9wAAAPZQTFRFmpqam5iYnJaWnJeXnpSUn5OTopCQpoqKpouLp4iIqIiIrYCAt3V1vW1tv2xsmZmZmpeXnpKS/x4e/x8f/yAg/yIi/yQk/yUl/yYm/ygo/ykp/yws/zAw/zIy/zMz/zQ0/zU1/zY2/zw8/0BA/0ZG/0pK/1FR/1JS/1NT/1RU/1VV/1ZW/1dX/1pa/15e/19f/2Zm/2lp/21t/25u/3R0/3p6/4CA/4GB/4SE/4iI/46O/4+P/52d/6am/6ur/66u/7Oz/7S0/7e3/87O/9fX/9zc/93d/+Dg/+vr/+3t/+/v//Dw//Ly//X1//f3//n5//z8////gzaKowAAAA90Uk5T/Pz8/Pz8/Pz8/Pz8/f39ppQKWQAAAAFiS0dEEnu8bAAAAACuSURBVAhbPY9ZF4FQFEZPSKbIMmWep4gMGTKLkIv6/3/GPbfF97b3w17rA0kQOPgvAeHW6uJ6+5h7HqLdwowgOzejXRXBdx6UdSru216xuOMBHHNU0clTzeSUA6EhF8V8kqroluMiU6HKcuf4phGPr1o2q9kYZWwNq1qfRRmTaXpqsyjj17KkWCxKBUBgXWueHIyiAIg18gsse4KHkLF5IKIY10WQgv7fOy4ST34BRiopZ8WLNrgAAAAASUVORK5CYII=);
  background-repeat: no-repeat;
  background-position: 0 .2em;
  padding-left: 1.5em;
}
.done0 {
  /* list-style: none; */
  background-image: url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA8AAAAPCAYAAAA71pVKAAAABHNCSVQICAgIfAhkiAAAAAlwSFlzAAAAxQAAAMUBHc26qAAAABl0RVh0U29mdHdhcmUAd3d3Lmlua3NjYXBlLm9yZ5vuPBoAAAA7SURBVCiR7dMxEgAgCANBI3yVRzF5KxNbW6wsuH7LQ2YKQK1mkswBVERYF5Os3UV3gwd/jF2SkXy66gAZkxS6BniubAAAAABJRU5ErkJggg==);
  background-repeat: no-repeat;
  background-position: 0 .2em;
  padding-left: 1.5em;
}
.done1 {
  background-image: url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA8AAAAPCAYAAAA71pVKAAAABHNCSVQICAgIfAhkiAAAAAlwSFlzAAAAxQAAAMUBHc26qAAAABl0RVh0U29mdHdhcmUAd3d3Lmlua3NjYXBlLm9yZ5vuPBoAAABtSURBVCiR1ZO7DYAwDER9BDmTeZQMFXmUbGYpOjrEryA0wOvO8itOslFrJYAug5BMM4BeSkmjsrv3aVTa8p48Xw1JSkSsWVUFwD05IqS1tmYzk5zzae9jnVVVzGyXb8sALjse+euRkEzu/uirFomVIdDGOLjuAAAAAElFTkSuQmCC);
  background-repeat: no-repeat;
  background-position: 0 .15em;
  padding-left: 1.5em;
}
.done2 {
  background-image: url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA8AAAAPCAYAAAA71pVKAAAABHNCSVQICAgIfAhkiAAAAAlwSFlzAAAAxQAAAMUBHc26qAAAABl0RVh0U29mdHdhcmUAd3d3Lmlua3NjYXBlLm9yZ5vuPBoAAAB1SURBVCiRzdO5DcAgDAVQGxjAYgTvxlDIu1FTIRYAp8qlFISkSH7l5kk+ZIwxKiI2mIyqWoeILYRgZ7GINDOLjnmF3VqklKCUMgTee2DmM661Qs55iI3Zm/1u5h9sm4ig9z4ERHTFzLyd4G4+nFlVrYg8+qoF/c0kdpeMsmcAAAAASUVORK5CYII=);
  background-repeat: no-repeat;
  background-position: 0 .15em;
  padding-left: 1.5em;
}
.done3 {
  background-image: url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA8AAAAPCAYAAAA71pVKAAAABHNCSVQICAgIfAhkiAAAAAlwSFlzAAAAxQAAAMUBHc26qAAAABl0RVh0U29mdHdhcmUAd3d3Lmlua3NjYXBlLm9yZ5vuPBoAAABoSURBVCiR7dOxDcAgDATA/0DtUdiKoZC3YhLkHjkVKF3idJHiztKfvrHZWnOSE8Fx95RJzlprimJVnXktvXeY2S0SEZRSAAAbmxnGGKH2I5T+8VfxPhIReQSuuY3XyYWa3T2p6quvOgGrvSFGlewuUAAAAABJRU5ErkJggg==);
  background-repeat: no-repeat;
  background-position: 0 .15em;
  padding-left: 1.5em;
}
.done4 {
  background-image: url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABIAAAAQCAYAAAAbBi9cAAAABHNCSVQICAgIfAhkiAAAAAlwSFlzAAAAzgAAAM4BlP6ToAAAABl0RVh0U29mdHdhcmUAd3d3Lmlua3NjYXBlLm9yZ5vuPBoAAAIISURBVDiNnZQ9SFtRFMd/773kpTaGJoQk1im4VDpWQcTNODhkFBcVTCNCF0NWyeDiIIiCm82QoIMIUkHUxcFBg1SEQoZszSat6cdTn1qNue92CMbEr9Sey+XC/Z/zu+f8h6ukUil3sVg0+M+4cFxk42/jH2wAqqqKSCSiPQdwcHHAnDHH9s/tN1h8V28ETdP+eU8fT9Nt62ancYdIPvJNtsu87bmjrJlrTDVM4RROJs1JrHPrD4Bar7A6cpc54iKOaTdJXCUI2UMVrQZ0Js7YPN18ECKkYNQcJe/OE/4dZsw7VqNXQMvHy3QZXQypQ6ycrtwDjf8aJ+PNEDSCzLpn7+m2pD8ZKHlKarYhy6XjEoCYGcN95qansQeA3fNdki+SaJZGTMQIOoL3W/Z89rxv+tokubNajlvk/vm+LFpF2XnUKZHI0I+QrI7Dw0OZTqdzUkpsM7mZTyfy5OPGyw1tK7AFSvmB/Ks8w8YwbUYbe6/3QEKv0vugfxWPnMLJun+d/kI/WLdizpNjMbAIKrhMF4OuwadBALqqs+RfInwUvuNi+fBd+wjogfogAFVRmffO02q01mZZ0HHdgXIzdz0QQLPezIQygX6llxNKKgOFARYCC49CqhoHIUTlss/Vx2phlYwjw8j1CAlfAiwQiJpiy7o1VHnsG5FISkoJu7Q/2YmmaV+i0ei7v38L2CBguSi5AAAAAElFTkSuQmCC);
  background-repeat: no-repeat;
  background-position: 0 .15em;
  padding-left: 1.5em;
}

code {
  font-family: Monaco, "Courier New", "DejaVu Sans Mono", "Bitstream Vera Sans Mono", monospace;
  -webkit-border-radius: 1px;
  -moz-border-radius: 1px;
  border-radius: 1px;
  -moz-background-clip: padding;
  -webkit-background-clip: padding-box;
  background-clip: padding-box;
  padding: 0px 3px;
  display: inline-block;
  color: #52595d;
  border: 1px solid #ccc;
  background-color: #f9f9f9;
}
"#;
