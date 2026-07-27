import PhotosUI
import SwiftUI
import Vision
import VisionKit

/// Scan or import *any* timetable: capture it with the document scanner, pick a
/// photo/screenshot, or paste text; Apple Vision reads it on-device, the server
/// turns the text into draft events, and the user reviews before importing.
struct ScanScheduleView: View {
    @EnvironmentObject var store: CalendarStore
    @Environment(\.dismiss) private var dismiss

    @State private var text = ""
    @State private var drafts: [DraftEvent] = []
    @State private var reviewing = false
    @State private var busy = false
    @State private var status: String?
    @State private var error: String?

    @State private var showScanner = false
    @State private var photoItem: PhotosPickerItem?

    var body: some View {
        NavigationStack {
            Group {
                if reviewing {
                    reviewList
                } else {
                    inputForm
                }
            }
            .navigationTitle(reviewing ? "Vérifier" : "Scanner un planning")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) { Button("Fermer") { dismiss() } }
            }
            .fullScreenCover(isPresented: $showScanner) {
                DocumentScanner { images in
                    showScanner = false
                    Task { await recognize(images) }
                } onCancel: { showScanner = false }
                .ignoresSafeArea()
            }
            .onChange(of: photoItem) { _, item in
                guard let item else { return }
                Task {
                    if let data = try? await item.loadTransferable(type: Data.self),
                       let img = UIImage(data: data) {
                        await recognize([img])
                    }
                    photoItem = nil
                }
            }
        }
    }

    // MARK: Input

    private var inputForm: some View {
        Form {
            Section {
                Text("Prends en photo n'importe quel emploi du temps (papier, écran, capture) ou colle son texte. L'app en extrait les événements, tu vérifies, puis tout est copié dans ton calendrier.")
                    .font(.footnote).foregroundStyle(.secondary)
            }

            Section {
                if VNDocumentCameraViewController.isSupported {
                    Button { showScanner = true } label: {
                        Label("Scanner avec l'appareil photo", systemImage: "doc.viewfinder")
                    }
                }
                PhotosPicker(selection: $photoItem, matching: .images) {
                    Label("Choisir une image / capture", systemImage: "photo")
                }
            }

            Section("Texte") {
                TextEditor(text: $text)
                    .frame(minHeight: 140)
                    .overlay(alignment: .topLeading) {
                        if text.isEmpty {
                            Text("Lundi 9h-10h30 Mathématiques\nMardi 14h Sport\n12/09 9h Rendez-vous médecin")
                                .font(.footnote).foregroundStyle(.tertiary)
                                .padding(.top, 8).padding(.leading, 5).allowsHitTesting(false)
                        }
                    }
            }

            if let status { Section { Text(status).font(.footnote).foregroundStyle(.secondary) } }
            if let error { Section { Text(error).font(.footnote).foregroundStyle(.red) } }

            Section {
                Button {
                    Task { await analyse() }
                } label: {
                    if busy { ProgressView() } else { Text("Analyser le texte").frame(maxWidth: .infinity) }
                }
                .disabled(busy || text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            }
        }
    }

    // MARK: Review

    private var reviewList: some View {
        Form {
            Section {
                Text("\(drafts.count) événement\(drafts.count > 1 ? "s" : "") détecté\(drafts.count > 1 ? "s" : ""). Décoche ceux à ignorer et corrige les titres si besoin.")
                    .font(.footnote).foregroundStyle(.secondary)
            }
            Section {
                ForEach($drafts) { $d in
                    HStack(alignment: .top, spacing: 10) {
                        Button {
                            d.keep.toggle()
                        } label: {
                            Image(systemName: d.keep ? "checkmark.circle.fill" : "circle")
                                .foregroundStyle(d.keep ? Color.accentColor : Color.secondary)
                        }
                        .buttonStyle(.plain)
                        VStack(alignment: .leading, spacing: 2) {
                            TextField("Titre", text: $d.title)
                            Text(label(d)).font(.caption).foregroundStyle(.secondary)
                        }
                    }
                    .opacity(d.keep ? 1 : 0.45)
                }
            }
            if let error { Section { Text(error).font(.footnote).foregroundStyle(.red) } }
            Section {
                Button {
                    Task { await importKept() }
                } label: {
                    if busy {
                        ProgressView().frame(maxWidth: .infinity)
                    } else {
                        Text("Importer \(keptCount) événement\(keptCount > 1 ? "s" : "")")
                            .frame(maxWidth: .infinity)
                    }
                }
                .disabled(busy || keptCount == 0)
                Button("‹ Retour") { reviewing = false }.disabled(busy)
            }
        }
    }

    private var keptCount: Int { drafts.filter(\.keep).count }

    private func label(_ d: DraftEvent) -> String {
        let f = DateFormatter()
        f.locale = Locale(identifier: "fr_FR")
        f.timeZone = TimeZone(identifier: "Europe/Paris")
        if d.allDay {
            f.dateFormat = "EEEE d MMM"
            return "\(f.string(from: d.startDate).capitalized) · toute la journée"
        }
        f.dateFormat = "EEEE d MMM · HH:mm"
        let day = f.string(from: d.startDate).capitalized
        f.dateFormat = "HH:mm"
        return "\(day)–\(f.string(from: d.endDate))"
    }

    // MARK: Actions

    private func recognize(_ images: [UIImage]) async {
        busy = true; error = nil
        status = "Lecture de l'image…"
        let recognized = await Self.ocr(images)
        await MainActor.run {
            if recognized.isEmpty {
                status = nil
                error = "Aucun texte lisible. Réessaie ou colle le texte à la main."
            } else {
                text = text.isEmpty ? recognized : text + "\n" + recognized
                status = "Image lue ✓ — vérifie le texte puis analyse-le."
            }
            busy = false
        }
    }

    private func analyse() async {
        busy = true; error = nil; status = nil
        do {
            let found = try await API.parseSchedule(text)
            await MainActor.run {
                if found.isEmpty {
                    error = "Aucun événement détecté. Ajoute des jours et des horaires (ex : « Lundi 9h-10h Maths »)."
                } else {
                    drafts = found
                    reviewing = true
                }
                busy = false
            }
        } catch {
            await MainActor.run { self.error = error.localizedDescription; busy = false }
        }
    }

    private func importKept() async {
        busy = true; error = nil
        let payloads = drafts.filter(\.keep).map { d in
            EventPayload(
                title: d.title.trimmingCharacters(in: .whitespaces).isEmpty ? "Sans titre" : d.title,
                description: nil, start: d.start, end: d.end, allDay: d.allDay,
                color: eventColors[4], recurrence: nil // violet marks imported events
            )
        }
        do {
            _ = try await API.createBatch(payloads)
            await store.refreshAll()
            await MainActor.run { dismiss() }
        } catch {
            await MainActor.run { self.error = error.localizedDescription; busy = false }
        }
    }

    /// On-device OCR (French), reading observations top-to-bottom, left-to-right.
    private static func ocr(_ images: [UIImage]) async -> String {
        await Task.detached(priority: .userInitiated) {
            var lines: [String] = []
            for image in images {
                guard let cg = image.cgImage else { continue }
                let request = VNRecognizeTextRequest()
                request.recognitionLevel = .accurate
                request.recognitionLanguages = ["fr-FR"]
                request.usesLanguageCorrection = true
                let handler = VNImageRequestHandler(cgImage: cg, options: [:])
                try? handler.perform([request])
                let obs = request.results ?? []
                let sorted = obs.sorted { a, b in
                    if abs(a.boundingBox.origin.y - b.boundingBox.origin.y) > 0.02 {
                        return a.boundingBox.origin.y > b.boundingBox.origin.y
                    }
                    return a.boundingBox.origin.x < b.boundingBox.origin.x
                }
                for o in sorted {
                    if let t = o.topCandidates(1).first?.string { lines.append(t) }
                }
            }
            return lines.joined(separator: "\n")
        }.value
    }
}

/// SwiftUI wrapper around VisionKit's document scanner (edge detection + crop).
struct DocumentScanner: UIViewControllerRepresentable {
    let onFinish: ([UIImage]) -> Void
    let onCancel: () -> Void

    func makeUIViewController(context: Context) -> VNDocumentCameraViewController {
        let vc = VNDocumentCameraViewController()
        vc.delegate = context.coordinator
        return vc
    }

    func updateUIViewController(_ vc: VNDocumentCameraViewController, context: Context) {}

    func makeCoordinator() -> Coordinator { Coordinator(self) }

    final class Coordinator: NSObject, VNDocumentCameraViewControllerDelegate {
        let parent: DocumentScanner
        init(_ parent: DocumentScanner) { self.parent = parent }

        func documentCameraViewController(_ controller: VNDocumentCameraViewController,
                                          didFinishWith scan: VNDocumentCameraScan) {
            var images: [UIImage] = []
            for i in 0..<scan.pageCount { images.append(scan.imageOfPage(at: i)) }
            parent.onFinish(images)
        }

        func documentCameraViewControllerDidCancel(_ controller: VNDocumentCameraViewController) {
            parent.onCancel()
        }

        func documentCameraViewController(_ controller: VNDocumentCameraViewController,
                                          didFailWithError error: Error) {
            parent.onCancel()
        }
    }
}
