# Struktur

*(Das kompositorische Rückgrat — wo diese Notiz im ganzen Werk sitzt)*

Constellation gibt Ihnen bereits acht **Denk-Links** — *supports, contradicts, causes, exemplifies, generalizes, derives-from, part-of, supersedes* — das Vokabular, mit dem Sie eine Idee zu einer anderen in Beziehung setzen. **Strukturelle Links** sind eine bewusst andere Art. Sie setzen nicht Idee zu Idee in Beziehung; sie legen die **geordnete Gestalt eines Werks** dar, das Sie aus Ihren Notizen aufbauen: Buch → Teil → Kapitel → Szene, oder jede beliebige Map-of-Content-Gliederung. Das **Struktur**-Panel ist der Ort, an dem Sie diese Gestalt lesen.

Die eine Frage, die die Struktur beantwortet, lautet: **„Wo sitzt diese Notiz im ganzen Werk?"** — *nicht* „wie verhält sich diese Idee zu jener." Diese zweite Frage gehört zu den Panels Backlinks und Outgoing Links, und die Struktur hält sich aus deren Weg.

---

## Warum strukturelle Links von Ihrem Denken getrennt gehalten werden

Eine strukturelle Platzierung ist **Autorschaft, keine zu bewertende Behauptung**. Eine Szene unter ein Kapitel zu setzen oder ein Kapitel unter ein Buch ist eine Entscheidung über die *Gestalt Ihres Manuskripts* — sie ist kein Beleg, kein Argument, nichts, dem widersprochen werden kann oder das mit der Zeit gewisser wird.

Daher sind strukturelle Links bewusst für jedes Denk-, Reife- und Verbindungsmaß unsichtbar:

- Sie zählen **nicht** als Verbindungen in den Backlinks oder Outgoing Links einer Notiz.
- Sie erhöhen **nicht** die Reife einer Notiz.
- Sie erscheinen **nicht** in der Sternenansicht oder im Graphen.

Ein Inhaltsverzeichnis sollte eine Notiz nicht „vernetzter" aussehen lassen, als sie ist. Ihre Denk-Links und die Gliederung Ihres Manuskripts sind zwei getrennte Dinge, und Constellation hält sie getrennt.

---

## Die zwei Arten — Sie tippen immer nur eine Seite

Sie deklarieren die Struktur von welchem Ende auch immer bequem ist, und Constellation ermittelt die Gegenrichtung für Sie. Sie müssen niemals beide Enden pflegen.

| Eigenschaft | Was sie bedeutet |
|---|---|
| **`parent`** | Der Platz *dieser Notiz* unter einem Elternteil. (Ein Kapitel gibt an, zu welchem Teil es gehört.) |
| **`contains`** | Die geordnete Liste der Kinder *dieser Notiz*. (Ein Buch listet seine Teile in der Lesereihenfolge auf.) |

Den `parent` eines Kindes zu deklarieren und es in einer `contains`-Liste aufzuführen sind zwei Wege, dasselbe zu sagen. Verwenden Sie, was zu Ihrer Denkweise passt — von oben nach unten (ein Buch, das seine Teile *enthält*) oder von unten nach oben (ein Kapitel, das seinen *Elternteil* benennt).

---

## Einen strukturellen Link anlegen — Schritt für Schritt

Sie legen die Struktur in den **Eigenschaften** einer Notiz an — dem Eigenschaften-Tab in der rechten Seitenleiste oder dem Eigenschaftenblock am Anfang der Notiz.

1. Klicken Sie auf **+ Eigenschaft hinzufügen**.
2. Tippen Sie als Schlüssel **`parent`** oder **`contains`**.
3. Tippen Sie als Wert den **Namen der Zielnotiz** — nur den Namen, zum Beispiel `Part I - The Cartographer`. **Sie tippen die eckigen Klammern nicht.** Constellation verpackt den Namen automatisch in einen `[[link]]`. (Wenn Sie einen Namen einfügen, der bereits Klammern enthält, wird er zu einem einzelnen `[[name]]` bereinigt — niemals zu einem doppelten `[[[ ]]]`.)
4. Für **`contains`** fügen Sie jedes Kind als eigenen Chip hinzu — tippen Sie einen Namen, drücken Sie Enter, tippen Sie den nächsten. **Die Reihenfolge, in der Sie sie hinzufügen, ist die Lesereihenfolge** der Gliederung.

> **Sie überstehen Umbenennungen sicher.** Benennen Sie ein Kapitel um, und sein Platz in der Struktur folgt automatisch — der Link zeigt auf die Notiz selbst, nicht auf ein eingefrorenes Stück Text. Sie müssen niemals eine Gliederung nach einer Umbenennung aufspüren und reparieren.

---

## Das Struktur-Panel lesen

Öffnen Sie den **Struktur**-Tab in der rechten Seitenleiste — direkt nach dem Backlinks-Tab.

- **Die Gliederung.** Überschrieben mit **OUTLINE** und einer Zählung, zeigt das Panel das **ganze Werk** als blaugrün-aufgezählten, eingerückten Baum — jeden Nachkommen des Werks, in Reihenfolge — nicht nur die eigenen Kinder der geöffneten Notiz. So sehen Sie, selbst wenn Sie auf einer einzelnen Szene stehen, das gesamte Buch um sie herum.
- **„Sie sind hier."** Die Notiz, die Sie gerade ansehen, ist innerhalb der Gliederung **hervorgehoben**, sodass Sie immer wissen, wo Sie stehen.
- **Der Breadcrumb.** Am oberen Rand zeigt ein blaugrüner Breadcrumb den Pfad das Rückgrat hinauf — zum Beispiel *The Atlas of Lost Places › Part I › Chapter 1*. Klicken Sie auf einen beliebigen Breadcrumb (oder auf eine beliebige Zeile in der Gliederung), um direkt zu dieser Notiz zu springen.
- **Ganzes Werk ⇄ Diese Notiz.** Ein Umschalter oben rechts wechselt zwischen dem gesamten Werk und nur dem eigenen Zweig der geöffneten Notiz. Er erscheint nur, wenn die Notiz einen Elternteil hat (andernfalls wären die beiden Ansichten identisch).

> **Eine Schleife hängt es niemals auf.** Wenn die Struktur versehentlich auf sich selbst zurückläuft — der Elternteil von Notiz A ist B, und der Elternteil von B ist A —, zeichnet die Gliederung die Kette und stoppt dann sauber, wobei sie die Schnittstelle mit einem kleinen **↻** markiert. Fahren Sie mit der Maus darüber für eine einzeilige Erklärung.

---

## Wenn zwei Notizen dasselbe Kind beanspruchen — „Umstritten"

Die Struktur soll ein sauberer Baum sein, daher sollte ein Kind genau einen Elternteil haben. Wenn zwei Notizen beide dasselbe Kind beanspruchen — eine über den eigenen **`parent`** des Kindes, die andere über ihre **`contains`**-Liste —, wählt Constellation **nicht** stillschweigend eine aus und verwirft die andere. Stattdessen wird diese Zeile als **Umstritten** gekennzeichnet, mit einem bernsteinfarbenen **⚠**-Badge, das den anderen Beanspruchenden benennt, sodass Sie den Konflikt sehen und entscheiden können.

Zwei Schaltflächen mit einem Klick lösen ihn:

- **Behalten** — den eigenen deklarierten Elternteil des Kindes behalten. (Diese Notiz gibt ihren Anspruch auf das Kind auf.)
- **Hierher verschieben** — diese Notiz als Elternteil akzeptieren. (Der `parent` des Kindes wechselt zu dieser Notiz.)

Beide Auswahlen aktualisieren die Notizdateien direkt und aktualisieren die Gliederung. **Nichts wird jemals ohne Ihren Klick geändert** — Constellation kennzeichnet den Konflikt und wartet auf Ihre Entscheidung.

---

## Gut zu wissen

- **Lokal und privat.** Die Gliederung wird bei Bedarf aus Ihren eigenen Notizen gelesen; nichts wird irgendwohin gesendet.
- **Schnell bei großen Werken.** Lange Gliederungen (über etwa 50 Zeilen) erhalten ihre eigene Bildlaufleiste und stellen nur die Zeilen auf dem Bildschirm dar, sodass ein großes Manuskript flüssig öffnet und scrollt.
- **Es spricht Ihre Sprache.** Die Beschriftungen des Panels, der Breadcrumb und die Lösungsschaltflächen erscheinen alle in Ihrer gewählten Oberflächensprache und spiegeln sich korrekt für Rechts-nach-links-Sprachen. Die Eigenschafts-*Schlüssel* `parent` / `contains` bleiben in der Datei in kanonischem Englisch (sodass die Struktur in jeder Sprache gleich gelesen wird), während ihre Pillen-Beschriftungen auf dem Bildschirm lokalisiert sind.
