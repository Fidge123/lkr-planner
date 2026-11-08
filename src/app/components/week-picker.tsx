import { ChevronLeft, ChevronRight } from "lucide-react";
import type { ReactNode, ToggleEvent } from "react";
import { useEffect, useId, useRef, useState } from "react";
import type { YearMonth } from "../util";
import {
  getIsoWeek,
  monthWeeks,
  pickerAnchorMonth,
  shiftMonth,
  toLocalISODate,
} from "../util";

const weekdayLabels = ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"];
const monthTitleFormat = new Intl.DateTimeFormat("de-DE", {
  month: "long",
  year: "numeric",
});
const dayLabelFormat = new Intl.DateTimeFormat("de-DE", {
  weekday: "long",
  day: "numeric",
  month: "long",
  year: "numeric",
});

export function WeekPicker({ weekDays, today, onJumpToWeek, children }: Props) {
  const id = useId();
  const triggerRef = useRef<HTMLButtonElement>(null);
  const pickerRef = useRef<HTMLElement>(null);
  const [isOpen, setIsOpen] = useState(false);
  const [shownMonth, setShownMonth] = useState(() =>
    pickerAnchorMonth(weekDays),
  );
  const displayedMonday = toLocalISODate(weekDays[0]);

  useEffect(() => {
    const picker = pickerRef.current;
    if (!isOpen || !picker) return;
    // Placed once on opening, so a scroll that moves the trigger must close it.
    const hide = () => picker.hidePopover();
    // The picker sits inside the grid's DOM, whose wheel listener would otherwise swipe the week under it.
    const keepWheel = (event: WheelEvent) => event.stopPropagation();
    document.addEventListener("scroll", hide, { capture: true, passive: true });
    picker.addEventListener("wheel", keepWheel, { passive: true });
    return () => {
      document.removeEventListener("scroll", hide, { capture: true });
      picker.removeEventListener("wheel", keepWheel);
    };
  }, [isOpen]);

  const handleBeforeToggle = (event: ToggleEvent<HTMLElement>) => {
    const isOpening = event.newState === "open";
    setIsOpen(isOpening);
    if (!isOpening) return;
    setShownMonth(pickerAnchorMonth(weekDays));
    const trigger = triggerRef.current?.getBoundingClientRect();
    if (!trigger) return;
    event.currentTarget.style.top = `${trigger.bottom + 4}px`;
    event.currentTarget.style.left = `${trigger.left}px`;
  };

  const choose = (day: Date) => {
    pickerRef.current?.hidePopover();
    onJumpToWeek(day);
  };

  return (
    <>
      <button
        ref={triggerRef}
        type="button"
        popoverTarget={id}
        aria-label={`KW ${getIsoWeek(weekDays[0])}, Woche auswählen`}
        className="btn btn-ghost flex h-auto w-fit -mx-2 -my-1 gap-1.5 border-0 px-2 py-1 leading-normal font-bold text-primary"
      >
        {children}
      </button>
      <section
        ref={pickerRef}
        id={id}
        popover="auto"
        onBeforeToggle={handleBeforeToggle}
        aria-label="Woche auswählen"
        className="card card-border fixed inset-auto m-0 flex-row gap-6 bg-base-100 p-4 font-normal text-base-content shadow-xl not-open:hidden"
      >
        <button
          type="button"
          aria-label="Vorheriger Monat"
          className="btn btn-ghost btn-sm btn-square absolute top-3 left-3"
          onClick={() => setShownMonth((month) => shiftMonth(month, -1))}
        >
          <ChevronLeft className="size-4" />
        </button>
        <button
          type="button"
          aria-label="Nächster Monat"
          className="btn btn-ghost btn-sm btn-square absolute top-3 right-3"
          onClick={() => setShownMonth((month) => shiftMonth(month, 1))}
        >
          <ChevronRight className="size-4" />
        </button>
        {[shiftMonth(shownMonth, -1), shownMonth].map((month) => (
          <MonthGrid
            key={`${month.year}-${month.month}`}
            month={month}
            displayedMonday={displayedMonday}
            today={today}
            onChoose={choose}
          />
        ))}
      </section>
    </>
  );
}

interface Props {
  weekDays: Date[];
  today: string;
  onJumpToWeek: (day: Date) => void;
  children: ReactNode;
}

function MonthGrid({
  month,
  displayedMonday,
  today,
  onChoose,
}: MonthGridProps) {
  return (
    <section className="flex flex-col gap-1">
      <h3 className="text-center font-semibold">
        {monthTitleFormat.format(new Date(month.year, month.month, 1))}
      </h3>
      <ul className="grid grid-cols-8 text-center text-xs text-base-content/60">
        <li>KW</li>
        {weekdayLabels.map((label) => (
          <li key={label}>{label}</li>
        ))}
      </ul>
      <ol className="flex flex-col gap-0.5">
        {monthWeeks(month.year, month.month).map(({ weekNumber, days }) => {
          const isDisplayed = toLocalISODate(days[0]) === displayedMonday;
          return (
            <li
              key={weekNumber}
              aria-current={isDisplayed || undefined}
              className={`grid grid-cols-8 rounded-field ${isDisplayed ? "bg-primary/10" : ""}`}
            >
              <button
                type="button"
                aria-label={`KW ${weekNumber}`}
                className="btn btn-ghost btn-sm btn-square font-normal text-base-content/60"
                onClick={() => onChoose(days[0])}
              >
                {weekNumber}
              </button>
              {days.map((day) => {
                const isToday = toLocalISODate(day) === today;
                return (
                  <button
                    key={day.getDate()}
                    type="button"
                    aria-label={dayLabelFormat.format(day)}
                    aria-current={isToday ? "date" : undefined}
                    className={dayClass(day, month, isToday)}
                    onClick={() => onChoose(day)}
                  >
                    {day.getDate()}
                  </button>
                );
              })}
            </li>
          );
        })}
      </ol>
    </section>
  );
}

interface MonthGridProps {
  month: YearMonth;
  displayedMonday: string;
  today: string;
  onChoose: (day: Date) => void;
}

function dayClass(day: Date, month: YearMonth, isToday: boolean): string {
  const size = "btn btn-sm btn-square font-normal";
  if (isToday) return `${size} btn-primary`;
  if (day.getMonth() !== month.month) {
    return `${size} btn-ghost text-base-content/40`;
  }
  return `${size} btn-ghost`;
}
