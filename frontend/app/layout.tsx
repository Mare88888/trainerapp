import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "CoachLift",
  description: "Next.js frontend with Rust backend API",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}

