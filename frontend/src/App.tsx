import { useCallback, useEffect, useMemo, useState } from "react";
import CalendarGrid from "./components/CalendarGrid";
import DayAgenda from "./components/DayAgenda";
import EventModal from "./components/EventModal";
import SearchModal from "./components/SearchModal";
import TideSpotsModal from "./components/TideSpotsModal";
import {
  createEvent,
  deleteEvent,
  fetchBeachWeather,
  fetchEvents,
  fetchPrefs,
  fetchState,
  importState,
  updateEvent,
} from "./api";
import { eventCoversDay, MONTH_NAMES, monthGridDays } from "./dates";
import { getSetting, loadLocal, MARKER_KEY, newMarker, saveLocal } from "./storage";
import { hasNativeReminders, syncNativeReminders } from "./notifications";
import { DEFAULT_PREFS } from "./types";
import type { BeachWeather, CalendarEvent, EventPayload, NotifPrefs } from "./types";

interface ModalState {
  event: CalendarEvent | null;
  initialDate: Date;
}

export default function App() {
  const now = new Date();
  const [year, setYear] = useState(now.getFullYear());
  const [month, setMonth] = useState(now.getMonth());
  const [selectedDay, setSelectedDay] = useState<Date>(now);
  const [events, setEvents] = useState<CalendarEvent[]>([]);
  const [weather, setWeather] = useState<BeachWeather[]>([]);
  const [prefs, setPrefs] = useState<NotifPrefs>(DEFAULT_PREFS);
  const [modal, setModal] = useState<ModalState | null>(null);
  const [tideModal, setTideModal] = useState(false);
  const [searchModal, setSearchModal] = useState(false);
  const [calCollapsed, setCalCollapsed] = useState(() => {
    try {
      return localStorage.getItem("calCollapsed") === "1";
    } catch {
      return false;
    }
  });
  const [voiceEnabled, setVoiceEnabledState] = useState(() => {
    try {
      // Visible by default; only hidden if the user turned it off.
      return localStorage.getItem("voiceEnabled") !== "0";
    } catch {
      return true;
    }
  });

  const setVoiceEnabled = (on: boolean) => {
    setVoiceEnabledState(on);
    try {
      localStorage.setItem("voiceEnabled", on ? "1" : "0");
    } catch {
      // storage unavailable: still works this session
    }
  };
  const [error, setError] = useState<string | null>(null);

  // Bounds of the visible grid (6 weeks), not just the month
  const [from, to] = useMemo(() => {
    const days = monthGridDays(year, month);
    const first = days[0];
    const afterLast = new Date(days[days.length - 1]);
    afterLast.setDate(afterLast.getDate() + 1);
    return [first.toISOString(), afterLast.toISOString()];
  }, [year, month]);

  const reload = useCallback(() => {
    fetchEvents(from, to)
      .then((evts) => {
        setEvents(evts);
        setError(null);
      })
      .catch((err) => setError(err instanceof Error ? err.message : "Erreur de chargement"));
    // Keep the on-device backup in step with the server. Only states that
    // carry the backup marker are stored: a marker-less snapshot (server
    // freshly booted, marker not written yet) must never overwrite a good
    // local copy — the marker is what loss detection compares.
    fetchState()
      .then((s) => {
        if (getSetting(s, MARKER_KEY)) saveLocal(s);
      })
      .catch(() => {});
  }, [from, to]);

  useEffect(reload, [reload]);

  // Device safety net: if the server rebooted with an empty database (its
  // backup marker is gone), push the phone's local copy back, then make
  // sure a marker exists and refresh the local copy.
  useEffect(() => {
    (async () => {
      try {
        let server = await fetchState();
        const local = loadLocal();
        const localMarker = local ? getSetting(local, MARKER_KEY) : null;
        if (local && localMarker && getSetting(server, MARKER_KEY) !== localMarker) {
          await importState(local);
          server = await fetchState();
          reload();
          reloadWeather();
        }
        if (!getSetting(server, MARKER_KEY)) {
          await importState({ settings: [{ key: MARKER_KEY, value: newMarker() }] });
          server = await fetchState();
        }
        saveLocal(server);
      } catch {
        // Offline or server unreachable: the safety net simply waits.
      }
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Beach weather for the selected spots (best-effort: silent on failure)
  const reloadWeather = useCallback(() => {
    fetchBeachWeather()
      .then(setWeather)
      .catch(() => setWeather([]));
  }, []);

  useEffect(reloadWeather, [reloadWeather]);

  // Recompute the weather every hour, and whenever the app comes back to the
  // foreground, so cities and beaches stay current without a manual refresh.
  useEffect(() => {
    const hourly = window.setInterval(reloadWeather, 3600_000);
    const onVisible = () => {
      if (document.visibilityState === "visible") reloadWeather();
    };
    document.addEventListener("visibilitychange", onVisible);
    return () => {
      window.clearInterval(hourly);
      document.removeEventListener("visibilitychange", onVisible);
    };
  }, [reloadWeather]);

  // Notification preferences (morning-briefing hour, reminder lead, toggles)
  const loadPrefs = useCallback(() => {
    fetchPrefs()
      .then(setPrefs)
      .catch(() => {});
  }, []);

  useEffect(loadPrefs, [loadPrefs]);

  // Native iOS reminders: fetch the next two weeks and let the shell schedule
  // local notifications (event reminders + morning briefing). No-op outside
  // the iOS webview.
  const syncReminders = useCallback(() => {
    if (!hasNativeReminders()) return;
    const now = Date.now();
    fetchEvents(new Date(now).toISOString(), new Date(now + 14 * 86400_000).toISOString())
      .then((evts) => syncNativeReminders(evts, weather, prefs))
      .catch(() => {});
  }, [weather, prefs]);

  useEffect(syncReminders, [syncReminders]);

  const shiftMonth = (delta: number) => {
    const d = new Date(year, month + delta, 1);
    setYear(d.getFullYear());
    setMonth(d.getMonth());
  };

  const goToday = () => {
    const d = new Date();
    setYear(d.getFullYear());
    setMonth(d.getMonth());
    setSelectedDay(d);
  };

  const selectDay = (day: Date) => {
    setSelectedDay(day);
    // Selecting a day of the previous/next month navigates there
    if (day.getMonth() !== month || day.getFullYear() !== year) {
      setYear(day.getFullYear());
      setMonth(day.getMonth());
    }
  };

  const save = async (payload: EventPayload) => {
    if (modal?.event) {
      await updateEvent(modal.event.id, payload);
    } else {
      await createEvent(payload);
    }
    setModal(null);
    reload();
    syncReminders();
  };

  const remove = async () => {
    if (modal?.event) {
      await deleteEvent(modal.event.id);
    }
    setModal(null);
    reload();
    syncReminders();
  };

  const toggleCalendar = () => {
    setCalCollapsed((v) => {
      const next = !v;
      try {
        localStorage.setItem("calCollapsed", next ? "1" : "0");
      } catch {
        // storage unavailable (private mode): still works this session
      }
      return next;
    });
  };

  const dayEvents = events.filter((ev) => eventCoversDay(ev.start, ev.end, selectedDay));

  return (
    <div className="app">
      <header className="toolbar">
        <button className="nav-btn" onClick={() => shiftMonth(-1)} aria-label="Mois précédent">
          ‹
        </button>
        <button className="month-label" onClick={goToday} title="Revenir à aujourd'hui">
          {MONTH_NAMES[month]} {year}
        </button>
        <div className="nav-group">
          <button className="nav-btn" onClick={() => shiftMonth(1)} aria-label="Mois suivant">
            ›
          </button>
          <button
            className="nav-btn"
            onClick={() => setSearchModal(true)}
            aria-label="Rechercher un événement"
          >
            🔍
          </button>
          <button
            className="nav-btn"
            onClick={() => setTideModal(true)}
            aria-label="Choisir plages (marées) et villes (météo)"
          >
            🌊
          </button>
        </div>
      </header>
      {error && <p className="error banner">⚠ {error}</p>}
      <CalendarGrid
        year={year}
        month={month}
        events={events}
        weather={weather}
        selectedDay={selectedDay}
        onSelectDay={selectDay}
        collapsed={calCollapsed}
        onToggleCollapse={toggleCalendar}
      />
      <DayAgenda
        day={selectedDay}
        events={dayEvents}
        weather={weather}
        voiceEnabled={voiceEnabled}
        onEventClick={(event) => setModal({ event, initialDate: new Date(event.start) })}
        onAdd={() => setModal({ event: null, initialDate: selectedDay })}
      />
      <button
        className="fab"
        aria-label="Nouvel événement"
        onClick={() => setModal({ event: null, initialDate: selectedDay })}
      >
        +
      </button>
      {modal && (
        <EventModal
          event={modal.event}
          initialDate={modal.initialDate}
          onSave={save}
          onDelete={remove}
          onClose={() => setModal(null)}
        />
      )}
      {searchModal && (
        <SearchModal
          onPick={(day) => {
            setSearchModal(false);
            selectDay(day);
          }}
          onClose={() => setSearchModal(false)}
        />
      )}
      {tideModal && (
        <TideSpotsModal
          voiceEnabled={voiceEnabled}
          onVoiceChange={setVoiceEnabled}
          onSaved={() => {
            setTideModal(false);
            reload();
            reloadWeather();
            loadPrefs();
          }}
          onClose={() => setTideModal(false)}
        />
      )}
    </div>
  );
}
