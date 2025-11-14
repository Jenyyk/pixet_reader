import javax.swing.*;
import java.awt.*;
import java.awt.image.BufferedImage;
import java.io.*;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;

/**
 * GridViewer
 *
 * Loads multiple frames from frame.txt (in the same directory as the program) with format like:
 * Frame { data: [[0, 100, 5000, ...], [200, 8000, 10000, ...], ...] } Frame { data: [[...]] } ...
 * Combines all frames by summing the grayscale values (clamped to 10000) and displays the resulting 256x256 grid.
 */
public class Main {

    public static void main(String[] args) {
        SwingUtilities.invokeLater(() -> {
            try {
                File file = new File("log.txt");
                if (!file.exists()) {
                    JOptionPane.showMessageDialog(null, "frame.txt not found in directory: " + System.getProperty("user.dir"));
                    System.exit(0);
                }

                int[][] matrix = loadAndCombineFrames(file);

                JFrame frame = new JFrame("Combined 256x256 Grayscale Grid Viewer");
                frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);

                GridPanel panel = new GridPanel(matrix);
                panel.setPreferredSize(new Dimension(512, 512));

                frame.add(panel);
                frame.pack();
                frame.setLocationRelativeTo(null);
                frame.setVisible(true);

            } catch (Exception ex) {
                ex.printStackTrace();
                JOptionPane.showMessageDialog(null, "Error: " + ex.getMessage());
            }
        });
    }

    /**
     * Loads and combines all frames from a file that contains multiple Frame { data: ... } entries.
     * Each frame is combined by summing and clamping values to 10000.
     */
    private static int[][] loadAndCombineFrames(File file) throws IOException {
        String text = Files.readString(file.toPath());

        // Split by occurrences of "Frame"
        String[] frameTexts = text.split("Frame\\s*\\{");

        int[][] combined = new int[256][256];
        boolean firstFrame = true;

        for (String frameText : frameTexts) {
            if (!frameText.contains("data")) continue; // skip empty pieces

            int start = frameText.indexOf("[[");
            int end = frameText.lastIndexOf("]]");
            if (start == -1 || end == -1) continue;

            String data = frameText.substring(start + 2, end);
            data = data.replace('[', ' ').replace(']', ' ').replace(',', ' ');
            String[] tokens = data.trim().split("\\s+");

            List<Integer> vals = new ArrayList<>();
            for (String t : tokens) {
                if (t.isEmpty()) continue;
                try {
                    int v = Integer.parseInt(t);
                    vals.add(Math.max(0, Math.min(10000, v))); // clamp to [0,10000]
                } catch (NumberFormatException ignore) {}
            }

            if (vals.size() < 256 * 256) continue; // skip incomplete frames

            int[][] current = new int[256][256];
            for (int r = 0; r < 256; r++) {
                for (int c = 0; c < 256; c++) {
                    current[r][c] = vals.get(r * 256 + c);
                }
            }

            if (firstFrame) {
                for (int r = 0; r < 256; r++) {
                    System.arraycopy(current[r], 0, combined[r], 0, 256);
                }
                firstFrame = false;
            } else {
                for (int r = 0; r < 256; r++) {
                    for (int c = 0; c < 256; c++) {
                        int sum = combined[r][c] + current[r][c];
                        combined[r][c] = Math.min(10000, sum); // cap at 10000
                    }
                }
            }
        }

        return combined;
    }

    static class GridPanel extends JPanel {
        private final BufferedImage baseImage;

        GridPanel(int[][] matrix) {
            baseImage = new BufferedImage(256, 256, BufferedImage.TYPE_INT_RGB);
            fillImage(matrix);
        }

        private void fillImage(int[][] matrix) {
            for (int r = 0; r < 256; r++) {
                for (int c = 0; c < 256; c++) {
                    int val = Math.max(0, Math.min(10000, matrix[r][c]));
                    int gray = (int) (Math.min((val) * 255, 255));
                    int rgb = new Color(gray, gray, gray).getRGB();
                    baseImage.setRGB(c, r, rgb);
                }
            }
        }

        @Override
        protected void paintComponent(Graphics g) {
            super.paintComponent(g);
            Graphics2D g2 = (Graphics2D) g.create();
            g2.setRenderingHint(RenderingHints.KEY_INTERPOLATION, RenderingHints.VALUE_INTERPOLATION_NEAREST_NEIGHBOR);
            g2.drawImage(baseImage, 0, 0, getWidth(), getHeight(), null);
            g2.dispose();
        }
    }
}
