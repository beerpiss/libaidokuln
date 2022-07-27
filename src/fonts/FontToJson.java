import javax.imageio.ImageIO;
import java.awt.Color;
import java.awt.Font;
import java.awt.FontMetrics;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.RenderingHints;
import java.awt.font.*;
import java.awt.geom.*;
import java.awt.image.*;
import java.io.*;
import java.util.*;

public class FontToJson {

    public static int[] letterData(String letter, Font font) {
        BufferedImage img = new BufferedImage(1, 1, BufferedImage.TYPE_4BYTE_ABGR);
        Graphics2D g = img.createGraphics();

        //Set the font to be used when drawing the string
        g.setFont(font);

        //Get the string visual bounds
        FontRenderContext frc = g.getFontMetrics().getFontRenderContext();
        Rectangle2D rect = font.getStringBounds(letter, frc);
        //Release resources
        g.dispose();

        //Then, we have to draw the string on the final image

        //Create a new image where to print the character
        img = new BufferedImage((int) Math.ceil(rect.getWidth()), (int) Math.ceil(rect.getHeight()), BufferedImage.TYPE_INT_ARGB);
        g = img.createGraphics();
        g.setColor(Color.black); //Otherwise the text would be white
        g.setFont(font);

        //Calculate x and y for that string
        FontMetrics fm = g.getFontMetrics();
        int x = 0;
        int y = fm.getAscent(); //getAscent() = baseline
        g.drawString(letter, x, y);

        //Release resources
        g.dispose();
        int[] data = new int[img.getHeight()*img.getWidth()];
        for(int i = 0; i < img.getHeight(); i++) {
            for(int j = 0; j < img.getWidth(); j++) {
                data[i*img.getWidth()+j] = ((img.getRGB(j, i)&0xff000000)>>24)&0xff;
            }
        }
        return data;
    }

    public static int fontHeight(Font font) {
        BufferedImage img = new BufferedImage(1, 1, BufferedImage.TYPE_INT_ARGB);
        Graphics2D g = img.createGraphics();

        //Set the font to be used when drawing the string
        g.setFont(font);

        //Get the string visual bounds
        FontRenderContext frc = g.getFontMetrics().getFontRenderContext();
        Rectangle2D rect = font.getStringBounds(" ", frc);
        //Release resources
        g.dispose();

        //Then, we have to draw the string on the final image

        //Create a new image where to print the character
        return (int) Math.ceil(rect.getHeight());
    }

    public static void main(String[] args) throws IOException {
        int[] fontSizes = { 18, 24, 30, 36 };
        String output = "palatino.rs";
        String fontName = "Palatino Linotype";

        FileWriter fw = new FileWriter(output);
        String constName = output.split("\\.")[0].replace("/", "").toUpperCase();
        fw.write("use crate::fonts::Font;");

        for (int size : fontSizes) {
            int[][] font = new int[127-32][];
            final Font fontf = new Font(fontName, Font.PLAIN, size);
            for (char c = 32; c < 127; c++) {
                font[c-32] = letterData(Character.toString(c), fontf);
            }


            fw.write("pub const " + constName + size + ":Font=Font{height:" + fontHeight(fontf) + ".0,font:[");
            for (int[] character : font) {
                fw.write("&");
                fw.write(Arrays.toString(character).replace("\s", ""));
                fw.write(",");
            }
            fw.write("]};");
        }
        fw.write("\n");
        fw.close();
    }

}
