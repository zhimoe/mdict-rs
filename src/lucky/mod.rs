use std::iter::Iterator;

use once_cell::sync::Lazy;
use rand::{thread_rng, Rng};
pub fn lucky_word() -> String {
    let random_index = thread_rng().gen_range(0..WORD_LIST.len());
    WORD_LIST[random_index].to_string()
}
static WORD_LIST: Lazy<Vec<&str>> = Lazy::new(|| STRING_LINES.lines().collect());
static STRING_LINES: &str = r#"abjure
abrogate
abstemious
acumen
antebellum
auspicious
belie
bellicose
bowdlerize
chicanery
chromosome
churlish
circumlocution
circumnavigate
deciduous
deleterious
diffident
enervate
enfranchise
epiphany
equinox
euro
evanescent
expurgate
facetious
fatuous
feckless
fiduciary
filibuster
gamete
gauche
gerrymander
hegemony
hemoglobin
homogeneous
hubris
hypotenuse
impeach
incognito
incontrovertible
inculcate
infrastructure
interpolate
irony
jejune
kinetic
kowtow
laissez faire
lexicon
loquacious
lugubrious
metamorphosis
mitosis
moiety
nanotechnology
nihilism
nomenclature
nonsectarian
notarize
obsequious
oligarchy
omnipotent
orthography
oxidize
parabola
paradigm
parameter
pecuniary
photosynthesis
plagiarize
plasma
polymer
precipitous
quasar
quotidian
recapitulate
reciprocal
reparation
respiration
sanguine
soliloquy
subjugate
suffragist
supercilious
tautology
taxonomy
tectonic
tempestuous
thermodynamics
totalitarian
unctuous
usurp
vacuous
vehement
vortex
winnow
wrought
xenophobe
yeoman
ziggurat
salient"#;
