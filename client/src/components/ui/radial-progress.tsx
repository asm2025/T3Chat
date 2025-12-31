import { cn } from "@/lib/utils";

interface RadialProgressProps {
    size?: number;
    className?: string;
}

export function RadialProgress({ size = 16, className }: RadialProgressProps) {
    const center = size / 2;
    const radius = (size - 2) / 2;
    const circumference = 2 * Math.PI * radius;
    // Show about 30% of the circle as an arc
    const strokeDasharray = `${circumference * 0.3} ${circumference}`;

    return (
        <svg
            width={size}
            height={size}
            className={cn("animate-spin", className)}
            viewBox={`0 0 ${size} ${size}`}
            xmlns="http://www.w3.org/2000/svg">
            <circle
                cx={center}
                cy={center}
                r={radius}
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeDasharray={strokeDasharray}
                strokeLinecap="round"
                opacity={0.3}
            />
        </svg>
    );
}

